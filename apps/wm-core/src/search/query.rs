use std::collections::HashMap;
use std::sync::atomic::Ordering as AtomicOrdering;
use std::sync::Arc;

use wm_constants::*;

use petgraph::Direction;

use crate::engine::{EngineState, GraphEdge, WikiPageMeta};
use wm_embed::{rrf_fusion, top_k_cosine, SearchMode};
use wm_search::recency_boost;
use wm_search::{post_rrf_rerank, Bm25Index, IndexedDoc, ScoreBreakdown};

pub struct QueryParams {
    pub query: String,
    pub r#type: String,
    pub mode: String,
    pub limit: usize,
    pub offset: usize,
    pub recency: bool,
}

#[derive(Clone, Debug)]
pub struct QueryResult {
    pub id: String,
    pub score: f64,
    pub r#type: String,
    pub page_type: String,
    pub page_type_rank: u8,
    pub centrality: usize,
    pub snippet: String,
    pub score_breakdown: Option<ScoreBreakdown>,
}

#[derive(Clone, Debug, Default)]
pub struct SearchResponse {
    pub results: Vec<QueryResult>,
    pub degraded: bool,
    pub warning: Option<String>,
}

impl SearchResponse {
    pub fn new(results: Vec<QueryResult>) -> Self {
        Self {
            results,
            degraded: false,
            warning: None,
        }
    }

    pub fn degraded(results: Vec<QueryResult>, warning: impl Into<String>) -> Self {
        Self {
            results,
            degraded: true,
            warning: Some(warning.into()),
        }
    }
}

/// Provenance-weighted graph centrality for a node (D2b): the edge-type-weighted
/// inbound sum, where each inbound edge's type priority is scaled by its
/// provenance factor — explicit 1.0, derived 0.5, ambiguous 0.25.
pub(crate) fn provenance_weighted_centrality(
    graph: &petgraph::stable_graph::StableGraph<WikiPageMeta, GraphEdge>,
    idx: petgraph::stable_graph::NodeIndex,
) -> f64 {
    graph
        .edges_directed(idx, Direction::Incoming)
        .map(|e| f64::from(e.weight().priority()) * e.weight().provenance_factor())
        .sum()
}

/// Ranking key for the single deterministic search comparator. Fields are the
/// tie-break tiers in priority order: text `score`, then provenance-weighted
/// graph `centrality`, then `page_type_rank`, then `id`.
pub(crate) struct RankKey<'a> {
    pub score: f64,
    pub centrality: f64,
    pub page_type_rank: u8,
    pub id: &'a str,
}

/// The one deterministic ranking comparator shared by the live search sort and
/// its regression tests. Orders by descending text score, then descending
/// provenance-weighted centrality, then descending page-type rank, then
/// ascending id so the total order is stable across runs.
pub(crate) fn rank_cmp(a: &RankKey, b: &RankKey) -> std::cmp::Ordering {
    b.score
        .partial_cmp(&a.score)
        .unwrap_or(std::cmp::Ordering::Equal)
        .then_with(|| {
            b.centrality
                .partial_cmp(&a.centrality)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .then_with(|| b.page_type_rank.cmp(&a.page_type_rank))
        .then_with(|| a.id.cmp(b.id))
}

pub fn run_unified_search(
    engine: &EngineState,
    params: &QueryParams,
) -> Result<SearchResponse, String> {
    if engine.bm25_index.load().total_docs == 0 || engine.stale_flag.load(AtomicOrdering::Acquire) {
        let root = engine
            .project_root
            .read()
            .map(|r| r.clone())
            .unwrap_or_else(|_| std::env::current_dir().unwrap_or_default());
        let wiki_dir = root.join(WM_DIR).join(WIKI_DIR);
        if wiki_dir.exists() {
            let sections = crate::graph::build_sections_from_wiki(&wiki_dir);
            engine.section_corpus.store(Arc::new(sections.clone()));
            let docs: Vec<IndexedDoc> = sections
                .iter()
                .map(crate::search::indexed_doc_from_section)
                .collect();
            engine.bm25_index.store(Arc::new(Bm25Index::build(docs)));
            engine.stale_flag.store(false, AtomicOrdering::Release);
        }
    }

    let embedder_loaded = engine.embedder.is_loaded();
    let degraded_warning =
        "Semantic search unavailable — ONNX model not loaded. Results are keyword-only.";

    let snap = engine.graph.load();
    let graph = &snap.0;
    let id_index = &snap.1;

    let search_pages = params.r#type == "all"
        || params.r#type == "page"
        || params.r#type == "task"
        || params.r#type == "memory";

    let config_guard = engine
        .config
        .read()
        .map_err(|e| format!("config lock poisoned: {}", e))?;
    let rrf_k = f64::from(config_guard.search.rrf_k);
    let recency_model = config_guard.search.scoring.recency_model;
    let recency_stability = f64::from(config_guard.search.scoring.recency_stability_days);
    drop(config_guard);

    let mode = match SearchMode::parse_loose(&params.mode) {
        SearchMode::Auto => SearchMode::auto_detect(&params.query),
        other => other,
    };

    let mut all_results: Vec<QueryResult> = Vec::new();
    let mut degraded = false;

    if search_pages {
        let (page_results, was_degraded): (Vec<QueryResult>, bool) = match mode {
            SearchMode::Auto | SearchMode::Keyword => {
                let bm25 = engine.bm25_index.load();
                let r = bm25.search(&params.query, params.limit);
                (
                    r.iter()
                        .map(|r| QueryResult {
                            id: r.id.clone(),
                            score: r.score,
                            snippet: r.snippet.clone(),
                            r#type: "page".into(),
                            page_type: String::new(),
                            page_type_rank: r.page_type_rank,
                            centrality: r.centrality,
                            score_breakdown: None,
                        })
                        .collect(),
                    false,
                )
            }
            SearchMode::Semantic => {
                if !embedder_loaded {
                    return Err("Semantic search unavailable: no embedding model loaded".into());
                }
                let vectors = engine.vector_store.snapshot();
                if vectors.is_empty() {
                    return Err("No embeddings indexed. Run index rebuild first.".into());
                }
                let query_vec = engine
                    .embedder
                    .embed(&params.query)
                    .map_err(|e| format!("Embedding failed: {}", e))?;
                let top_k = top_k_cosine(&query_vec.0, &vectors, params.limit);
                (
                    top_k
                        .into_iter()
                        .map(|(id, score)| QueryResult {
                            id,
                            score,
                            snippet: String::new(),
                            r#type: "page".into(),
                            page_type: String::new(),
                            page_type_rank: 0,
                            centrality: 0,
                            score_breakdown: None,
                        })
                        .collect(),
                    false,
                )
            }
            SearchMode::Hybrid => {
                if !embedder_loaded {
                    let bm25 = engine.bm25_index.load();
                    let r = bm25.search(&params.query, params.limit);
                    (
                        r.iter()
                            .map(|r| QueryResult {
                                id: r.id.clone(),
                                score: r.score,
                                snippet: r.snippet.clone(),
                                r#type: "page".into(),
                                page_type: String::new(),
                                page_type_rank: r.page_type_rank,
                                centrality: r.centrality,
                                score_breakdown: None,
                            })
                            .collect(),
                        true,
                    )
                } else {
                    let bm25 = engine.bm25_index.load();
                    let bm25_results = bm25.search(
                        &params.query,
                        params.limit.checked_mul(2).unwrap_or(params.limit),
                    );
                    let bm25_pairs: Vec<(String, f64)> = bm25_results
                        .iter()
                        .map(|r| (r.id.clone(), r.score))
                        .collect();

                    let vectors = engine.vector_store.snapshot();

                    let query_vec = match engine.embedder.embed(&params.query) {
                        Ok(v) => v,
                        Err(_) => {
                            let bm25 = engine.bm25_index.load();
                            let r = bm25.search(&params.query, params.limit);
                            return Ok(SearchResponse::degraded(
                                r.iter()
                                    .map(|r| QueryResult {
                                        id: r.id.clone(),
                                        score: r.score,
                                        snippet: r.snippet.clone(),
                                        r#type: "page".into(),
                                        page_type: String::new(),
                                        page_type_rank: r.page_type_rank,
                                        centrality: r.centrality,
                                        score_breakdown: None,
                                    })
                                    .collect(),
                                "Semantic search unavailable — embedding failed. Results are keyword-only.",
                            ));
                        }
                    };

                    let mut semantic_pairs: Vec<(String, f64)> = if vectors.is_empty() {
                        Vec::new()
                    } else {
                        top_k_cosine(
                            &query_vec.0,
                            &vectors,
                            params.limit.checked_mul(2).unwrap_or(params.limit),
                        )
                    };

                    let turso_results =
                        engine.vector_store.search_turso(&query_vec.0, params.limit);
                    for (id, score) in turso_results {
                        if !semantic_pairs.iter().any(|(sid, _)| sid == &id) {
                            semantic_pairs.push((id, f64::from(score)));
                        }
                    }

                    let mut fused = rrf_fusion(&bm25_pairs, &semantic_pairs, rrf_k);
                    let query_tokens = wm_search::tokenize(&params.query);
                    let mut breakdowns =
                        post_rrf_rerank(&mut fused, &bm25.docs, &params.query, &query_tokens);

                    fused
                        .sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

                    let bm25_map: HashMap<&str, f64> =
                        bm25_pairs.iter().map(|(id, s)| (id.as_str(), *s)).collect();
                    let sem_map: HashMap<&str, f64> = semantic_pairs
                        .iter()
                        .map(|(id, s)| (id.as_str(), *s))
                        .collect();
                    for (id, bd) in breakdowns.iter_mut() {
                        if let Some(&bm25_s) = bm25_map.get(id.as_str()) {
                            bd.bm25 = bm25_s;
                        }
                        if let Some(&sem_s) = sem_map.get(id.as_str()) {
                            bd.semantic = sem_s;
                        }
                    }

                    let truncated: Vec<_> = fused.into_iter().take(params.limit).collect();
                    (
                        truncated
                            .into_iter()
                            .map(|(id, score)| {
                                let sb = breakdowns.remove(&id);
                                QueryResult {
                                    id,
                                    score,
                                    snippet: String::new(),
                                    r#type: "page".into(),
                                    page_type: String::new(),
                                    page_type_rank: 0,
                                    centrality: 0,
                                    score_breakdown: sb,
                                }
                            })
                            .collect(),
                        false,
                    )
                }
            }
        };
        degraded |= was_degraded;

        for mut r in page_results {
            let id = r.id.clone();
            let base_id = id.split('#').next().unwrap_or(&id);

            if let Some(&idx) = id_index.get(base_id) {
                let meta = &graph[idx];
                r.page_type = meta.page_type.as_str().to_string();
                r.centrality = meta.relates_to.len();
                r.page_type_rank = meta.page_type.priority_rank();
            }

            if r.score_breakdown.is_none() {
                r.score_breakdown = Some(ScoreBreakdown {
                    bm25: r.score,
                    rrf: 0.0,
                    semantic: 0.0,
                    title_density: 0.0,
                    exact_title: 0.0,
                    title_starts_with: 0.0,
                    title_contains: 0.0,
                    tag_overlap: 0.0,
                    exact_id: 0.0,
                    recency: 0.0,
                    final_score: 0.0,
                });
            }

            if params.recency && r.page_type == "task" {
                let days_since = if let Some(&idx) = id_index.get(&id) {
                    let meta = &graph[idx];
                    use chrono::NaiveDate;
                    if let Ok(d) = NaiveDate::parse_from_str(&meta.updated_at, "%Y-%m-%d") {
                        let updated = d
                            .and_hms_opt(0, 0, 0)
                            .map(|dt| dt.and_utc())
                            .unwrap_or_else(chrono::Utc::now);
                        let duration = chrono::Utc::now().signed_duration_since(updated);
                        (duration.num_hours() as f64 / 24.0).max(0.0)
                    } else {
                        DEFAULT_MEMORY_STABILITY_DAYS
                    }
                } else {
                    DEFAULT_MEMORY_STABILITY_DAYS
                };
                let recency_val = recency_boost(days_since, &recency_model, recency_stability);
                r.score *= recency_val;
                if let Some(ref mut bd) = r.score_breakdown {
                    bd.recency = recency_val;
                    bd.final_score = r.score;
                }
            } else if let Some(ref mut bd) = r.score_breakdown {
                bd.recency = 1.0;
                bd.final_score = r.score;
            }

            all_results.push(r);
        }
    }

    let mut weighted_centrality: HashMap<String, f64> = HashMap::new();
    for r in &all_results {
        let base_id = r.id.split('#').next().unwrap_or(&r.id);
        if let Some(&idx) = id_index.get(base_id) {
            weighted_centrality.insert(r.id.clone(), provenance_weighted_centrality(graph, idx));
        }
    }

    all_results.sort_by(|a, b| {
        let a_key = RankKey {
            score: a.score,
            centrality: weighted_centrality.get(&a.id).copied().unwrap_or(0.0),
            page_type_rank: a.page_type_rank,
            id: &a.id,
        };
        let b_key = RankKey {
            score: b.score,
            centrality: weighted_centrality.get(&b.id).copied().unwrap_or(0.0),
            page_type_rank: b.page_type_rank,
            id: &b.id,
        };
        rank_cmp(&a_key, &b_key)
    });
    all_results.truncate(params.limit);

    let offset = params.offset.min(all_results.len().saturating_sub(1));
    let final_results: Vec<QueryResult> = all_results.into_iter().skip(offset).collect();

    if degraded {
        Ok(SearchResponse::degraded(final_results, degraded_warning))
    } else {
        Ok(SearchResponse::new(final_results))
    }
}

pub fn merge_results_by_rrf(results: Vec<QueryResult>, k: f64, limit: usize) -> Vec<QueryResult> {
    let mut by_type: HashMap<String, Vec<&QueryResult>> = HashMap::new();
    for r in &results {
        by_type.entry(r.r#type.clone()).or_default().push(r);
    }

    let mut rrf_scores: HashMap<String, f64> = HashMap::new();
    for typed_results in by_type.values() {
        for (rank, r) in typed_results.iter().enumerate() {
            let score = 1.0 / (k + rank as f64);
            *rrf_scores.entry(r.id.clone()).or_insert(0.0) += score;
        }
    }

    let mut ranked: Vec<(f64, QueryResult)> = results
        .into_iter()
        .map(|r| {
            let rrf_score = rrf_scores.get(&r.id).copied().unwrap_or(0.0);
            (rrf_score, r)
        })
        .collect();
    ranked.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
    ranked.truncate(limit);
    ranked
        .into_iter()
        .map(|(score, mut r)| {
            r.score = score;
            r
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::EdgeProvenance;
    use crate::graph::build_graph_from_wiki;
    use petgraph::stable_graph::{NodeIndex, StableGraph};
    use tempfile::TempDir;
    use wm_search::SearchResult;

    fn fixture_wiki() -> (
        TempDir,
        StableGraph<WikiPageMeta, GraphEdge>,
        HashMap<String, NodeIndex>,
    ) {
        let tmp = TempDir::new().unwrap();
        let wiki_dir = tmp.path().join(".wm").join("wiki");
        std::fs::create_dir_all(wiki_dir.join("concepts")).unwrap();
        std::fs::create_dir_all(wiki_dir.join("patterns")).unwrap();

        std::fs::write(
            wiki_dir.join("concepts/cent-source.md"),
            "See @wiki/concepts/cent-explicit for the explicit target.\n",
        )
        .unwrap();
        std::fs::write(
            wiki_dir.join("concepts/cent-explicit.md"),
            "# Cent Explicit\n\nPlain page.\n",
        )
        .unwrap();
        std::fs::write(
            wiki_dir.join("concepts/cent-ambig.md"),
            "# Cent Ambig\n\nFirst ambiguous candidate.\n",
        )
        .unwrap();
        std::fs::write(
            wiki_dir.join("patterns/cent-ambig.md"),
            "# Cent Ambig\n\nSecond ambiguous candidate.\n",
        )
        .unwrap();
        std::fs::write(
            wiki_dir.join("concepts/cent-ref.md"),
            r#"---
relates_to:
  - type: references
    target: cent-ambig
---

Deliberately ambiguous short target.
"#,
        )
        .unwrap();

        let (graph, id_index) = build_graph_from_wiki(wiki_dir.as_path(), &[]);
        (tmp, graph, id_index)
    }

    #[test]
    fn ambiguous_edge_contributes_quarter_of_explicit_to_centrality() {
        let (_tmp, graph, id_index) = fixture_wiki();

        let explicit_idx = id_index
            .get("wiki:concepts:cent-explicit")
            .copied()
            .unwrap();
        let ambiguous_idx = {
            let mut found = None;
            for edge_idx in graph.edge_indices() {
                if graph[edge_idx].provenance == EdgeProvenance::Ambiguous {
                    found = Some(graph.edge_endpoints(edge_idx).unwrap().1);
                    break;
                }
            }
            found.expect("fixture must contain an ambiguous edge")
        };

        let explicit_cent = provenance_weighted_centrality(&graph, explicit_idx);
        let ambiguous_cent = provenance_weighted_centrality(&graph, ambiguous_idx);
        assert_eq!(
            explicit_cent, 1.0,
            "single explicit references edge (priority 1) must contribute 1.0"
        );
        assert!(
            (ambiguous_cent - 0.25).abs() < 1e-9,
            "ambiguous edge must contribute 0.25x an explicit edge, got {ambiguous_cent}"
        );

        let mut results = vec![
            SearchResult {
                id: graph[ambiguous_idx].id.clone(),
                score: 0.5,
                snippet: String::new(),
                page_type_rank: 4,
                centrality: 0,
            },
            SearchResult {
                id: graph[explicit_idx].id.clone(),
                score: 0.5,
                snippet: String::new(),
                page_type_rank: 4,
                centrality: 0,
            },
        ];
        results.sort_by(|a, b| {
            let wa = id_index
                .get(&a.id)
                .map(|&i| provenance_weighted_centrality(&graph, i))
                .unwrap_or(0.0);
            let wb = id_index
                .get(&b.id)
                .map(|&i| provenance_weighted_centrality(&graph, i))
                .unwrap_or(0.0);
            rank_cmp(
                &RankKey {
                    score: a.score,
                    centrality: wa,
                    page_type_rank: a.page_type_rank,
                    id: &a.id,
                },
                &RankKey {
                    score: b.score,
                    centrality: wb,
                    page_type_rank: b.page_type_rank,
                    id: &b.id,
                },
            )
        });
        assert_eq!(
            results[0].id, graph[explicit_idx].id,
            "explicit edge must outrank ambiguous edge at equal text score"
        );
        assert_eq!(results[1].id, graph[ambiguous_idx].id);
    }

    #[test]
    fn all_explicit_edges_are_neutral_for_centrality() {
        let tmp = TempDir::new().unwrap();
        let wiki_dir = tmp.path().join(".wm").join("wiki");
        std::fs::create_dir_all(wiki_dir.join("concepts")).unwrap();
        std::fs::create_dir_all(wiki_dir.join("patterns")).unwrap();

        std::fs::write(
            wiki_dir.join("concepts/a.md"),
            r#"---
relates_to:
  - type: references
    target: wiki:patterns:b
---

A.
"#,
        )
        .unwrap();
        std::fs::write(
            wiki_dir.join("patterns/b.md"),
            r#"---
relates_to:
  - type: references
    target: wiki:concepts:a
---

B.
"#,
        )
        .unwrap();

        let (graph, _id_index) = build_graph_from_wiki(wiki_dir.as_path(), &[]);
        for edge_idx in graph.edge_indices() {
            assert_eq!(graph[edge_idx].provenance, EdgeProvenance::Explicit);
        }

        for idx in graph.node_indices() {
            let raw: f64 = graph
                .edges_directed(idx, petgraph::Direction::Incoming)
                .map(|e| f64::from(e.weight().priority()))
                .sum();
            let weighted = provenance_weighted_centrality(&graph, idx);
            assert_eq!(
                weighted, raw,
                "with all edges explicit the provenance factor must be neutral"
            );
        }
    }
}
