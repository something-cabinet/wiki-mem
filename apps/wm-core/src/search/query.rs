//! Unified search query — orchestrates BM25, semantic, and hybrid searches
//! across wiki pages and memory entries, with RRF fusion and enrichment.

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::Ordering as AtomicOrdering;

use petgraph::Direction;

use wm_search::{Bm25Index, Field, IndexedDoc, ScoreBreakdown, SearchResult, post_rrf_rerank};
use wm_search::recency_boost;
use wm_embed::{rrf_fusion, top_k_cosine, SearchMode};
use crate::engine::{EdgeType, EngineState, WikiPageMeta};

// ─── Unified Query API ───────────────────────────────────────

/// Parameters for the unified search query.
pub struct QueryParams {
    pub query: String,
    pub r#type: String,   // "all", "page", "task", "memory"
    pub mode: String,      // "auto", "keyword", "semantic", "hybrid"
    pub limit: usize,      // default 10
    pub offset: usize,     // default 0
    pub recency: bool,     // apply recency boost to tasks
}

/// A single result from the unified search.
#[derive(Clone, Debug)]
pub struct QueryResult {
    pub id: String,
    pub score: f64,
    pub r#type: String,        // "page" or "memory"
    pub page_type: String,     // e.g., "task", "concept"
    pub page_type_rank: u8,
    pub centrality: usize,
    pub snippet: String,
    pub score_breakdown: Option<ScoreBreakdown>,
}

/// The unified search response, wrapping results with degraded-mode metadata.
#[derive(Clone, Debug)]
pub struct SearchResponse {
    pub results: Vec<QueryResult>,
    /// When true, semantic search was unavailable and results are keyword-only.
    pub degraded: bool,
    /// Human-readable explanation when degraded is true.
    pub warning: Option<String>,
}

impl SearchResponse {
    /// Create a normal (non-degraded) response from results.
    pub fn new(results: Vec<QueryResult>) -> Self {
        Self { results, degraded: false, warning: None }
    }

    /// Create a degraded response — semantic search was unavailable,
    /// results are keyword-only.
    pub fn degraded(results: Vec<QueryResult>, warning: impl Into<String>) -> Self {
        Self { results, degraded: true, warning: Some(warning.into()) }
    }
}

impl Default for SearchResponse {
    fn default() -> Self {
        Self { results: Vec::new(), degraded: false, warning: None }
    }
}

/// Enrich search results with graph centrality and page type rank, then re-sort.
/// Sort order: score desc → centrality desc → page_type_rank desc → id alpha
pub fn enrich_search_results_from_graph(
    results: &mut [SearchResult],
    graph: &petgraph::stable_graph::StableGraph<WikiPageMeta, EdgeType>,
    id_index: &HashMap<String, petgraph::stable_graph::NodeIndex>,
) {
    for r in results.iter_mut() {
        // Find page in graph
        if let Some(&idx) = id_index.get(&r.id) {
            let meta = &graph[idx];
            r.centrality = graph
                .edges_directed(idx, Direction::Incoming)
                .map(|e| e.weight().priority())
                .sum::<u8>() as usize;
            r.page_type_rank = meta.page_type.priority_rank();
        }
    }

    results.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| b.centrality.cmp(&a.centrality))
            .then_with(|| b.page_type_rank.cmp(&a.page_type_rank))
            .then_with(|| a.id.cmp(&b.id))
    });
}

/// Run a unified search across pages and/or memory using the engine indexes.
/// Returns results sorted by score (or RRF-fused when both types searched),
/// with enrichment, recency boost, memory salience, and offset applied.
/// When semantic search is unavailable in hybrid mode, returns keyword-only
/// results with `degraded: true` and a warning message instead of erroring.
pub fn run_unified_search(
    engine: &EngineState,
    params: &QueryParams,
) -> Result<SearchResponse, String> {
    // Auto-rebuild if BM25 is empty or stale flag is set
    if engine.bm25_index.load().total_docs == 0 || engine.stale_flag.load(AtomicOrdering::Acquire) {
        let root = engine
            .project_root
            .read()
            .map(|r| r.clone())
            .unwrap_or_else(|_| std::env::current_dir().unwrap_or_default());
        let wiki_dir = root.join(".wm").join("wiki");
        if wiki_dir.exists() {
            let sections = crate::graph::build_sections_from_wiki(&wiki_dir);
            engine.section_corpus.store(Arc::new(sections.clone()));
            let docs: Vec<IndexedDoc> = sections
                .iter()
                .map(|s| IndexedDoc {
                    id: s.section_id.clone(),
                    fields: vec![
                        Field::new("header", &s.header, 4.0),
                        Field::new("body", &s.body, 1.0),
                        Field::new("id", &s.section_id, 0.0),
                        Field::new("title", &s.title, 0.0),
                        Field::new("tags", &s.tags.join(" "), 0.0),
                    ],
                })
                .collect();
            engine.bm25_index.store(Arc::new(Bm25Index::build(docs)));
            engine.stale_flag.store(false, AtomicOrdering::Release);
        }
    }

    let embedder_loaded = engine.embedder.is_loaded();
    let degraded_warning = "Semantic search unavailable — ONNX model not loaded. Results are keyword-only.";

    // Snapshot for enrichment
    let snap = engine.graph.load();
    let graph = &snap.0;
    let id_index = &snap.1;

    // Memory is now indexed as regular pages, so all type filters search the same index.
    let search_pages = params.r#type == "all" || params.r#type == "page" || params.r#type == "task" || params.r#type == "memory";

    // Acquire config values
    let config_guard = engine
        .config
        .read()
        .map_err(|e| format!("config lock poisoned: {}", e))?;
    let rrf_k = config_guard.search.rrf_k as f64;
    let recency_model = config_guard.search.scoring.recency_model.clone();
    let recency_stability = config_guard.search.scoring.recency_stability_days as f64;
    drop(config_guard);

    // Determine search mode
    // Resolve auto-detection before matching to avoid unreachable pattern
    let mode = match SearchMode::from_str(&params.mode) {
        SearchMode::Auto => SearchMode::auto_detect(&params.query),
        other => other,
    };

    let mut all_results: Vec<QueryResult> = Vec::new();
    let mut degraded = false;

    // 1. Search pages
    if search_pages {
        let (page_results, was_degraded): (Vec<QueryResult>, bool) = match mode {
            SearchMode::Auto | SearchMode::Keyword => {
                let bm25 = engine.bm25_index.load();
                let r = bm25.search(&params.query, params.limit);
                (r.iter()
                    .map(|r| QueryResult {
                        id: r.id.clone(),
                        score: r.score,
                        snippet: r.snippet.clone(),
                        r#type: "page".to_string(),
                        page_type: String::new(),
                        page_type_rank: r.page_type_rank,
                        centrality: r.centrality,
                        score_breakdown: None,
                    })
                    .collect(), false)
            }
            SearchMode::Semantic => {
                if !embedder_loaded {
                    return Err(
                        "Semantic search unavailable: no embedding model loaded".to_string(),
                    );
                }
                let vectors = engine.vector_store.snapshot();
                if vectors.is_empty() {
                    return Err("No embeddings indexed. Run index rebuild first.".to_string());
                }
                let query_vec = engine
                    .embedder
                    .embed(&params.query)
                    .map_err(|e| format!("Embedding failed: {}", e))?;
                let top_k =
                    top_k_cosine(&query_vec.0, &vectors, params.limit);
                (top_k
                    .into_iter()
                    .map(|(id, score)| QueryResult {
                        id,
                        score,
                        snippet: String::new(),
                        r#type: "page".to_string(),
                        page_type: String::new(),
                        page_type_rank: 0,
                        centrality: 0,
                        score_breakdown: None,
                    })
                    .collect(), false)
            }
            SearchMode::Hybrid => {
                if !embedder_loaded {
                    // Embedder not available — fall back to keyword-only with degraded flag
                    let bm25 = engine.bm25_index.load();
                    let r = bm25.search(&params.query, params.limit);
                    (r.iter()
                        .map(|r| QueryResult {
                            id: r.id.clone(),
                            score: r.score,
                            snippet: r.snippet.clone(),
                            r#type: "page".to_string(),
                            page_type: String::new(),
                            page_type_rank: r.page_type_rank,
                            centrality: r.centrality,
                            score_breakdown: None,
                        })
                        .collect(), true)
                } else {
                    let bm25 = engine.bm25_index.load();
                    let bm25_results = bm25.search(&params.query, params.limit * 2);
                    let bm25_pairs: Vec<(String, f64)> = bm25_results
                        .iter()
                        .map(|r| (r.id.clone(), r.score))
                        .collect();

                    let vectors = engine.vector_store.snapshot();

                    // Try embedding — on failure fall back to BM25-only with degraded flag
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
                                        r#type: "page".to_string(),
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

                    // Search in-memory vectors
                    let mut semantic_pairs: Vec<(String, f64)> = if vectors.is_empty() {
                        Vec::new()
                    } else {
                        top_k_cosine(&query_vec.0, &vectors, params.limit * 2)
                    };

                    // Also search turso for additional results
                    let turso_results = engine.vector_store.search_turso(&query_vec.0, params.limit);
                    for (id, score) in turso_results {
                        // Only add if not already present (avoid duplicates)
                        if !semantic_pairs.iter().any(|(sid, _)| sid == &id) {
                            semantic_pairs.push((id, score as f64));
                        }
                    }

                    let mut fused =
                        rrf_fusion(&bm25_pairs, &semantic_pairs, rrf_k);
                    // Post-RRF rerank boosts applied to fused scores
                    let query_tokens = wm_search::tokenize(&params.query);
                    let mut breakdowns = post_rrf_rerank(&mut fused, &bm25.docs, &query_tokens);

                    // Fill in BM25 and semantic raw scores into breakdowns
                    let bm25_map: HashMap<&str, f64> = bm25_pairs.iter().map(|(id, s)| (id.as_str(), *s)).collect();
                    let sem_map: HashMap<&str, f64> = semantic_pairs.iter().map(|(id, s)| (id.as_str(), *s)).collect();
                    for (id, bd) in breakdowns.iter_mut() {
                        if let Some(&bm25_s) = bm25_map.get(id.as_str()) {
                            bd.bm25 = bm25_s;
                        }
                        if let Some(&sem_s) = sem_map.get(id.as_str()) {
                            bd.semantic = sem_s;
                        }
                    }

                    let truncated: Vec<_> = fused.into_iter().take(params.limit).collect();
                    (truncated
                        .into_iter()
                        .map(|(id, score)| {
                            let sb = breakdowns.remove(&id);
                            QueryResult {
                                id,
                                score,
                                snippet: String::new(),
                                r#type: "page".to_string(),
                                page_type: String::new(),
                                page_type_rank: 0,
                                centrality: 0,
                                score_breakdown: sb,
                            }
                        })
                        .collect(), false)
                }
            }
        };
        degraded |= was_degraded;

        // Enrich with page type info and apply recency boost
        for mut r in page_results {
            let id = r.id.clone();

            // Enrich from graph
            if let Some(&idx) = id_index.get(&id) {
                let meta = &graph[idx];
                r.page_type = meta.page_type.as_str().to_string();
                r.centrality = meta.relates_to.len();
                r.page_type_rank = meta.page_type.priority_rank();
            }

            // Ensure a breakdown exists (for keyword/semantic paths where it wasn't set yet)
            if r.score_breakdown.is_none() {
                r.score_breakdown = Some(ScoreBreakdown {
                    bm25: r.score,
                    rrf: 0.0,
                    semantic: 0.0,
                    title_density: 0.0,
                    exact_title: 0.0,
                    tag_overlap: 0.0,
                    exact_id: 0.0,
                    recency: 0.0,
                    final_score: 0.0,
                });
            }

            // Recency boost for tasks
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
                        7.0
                    }
                } else {
                    7.0
                };
                let recency_val =
                    recency_boost(days_since, &recency_model, recency_stability);
                r.score *= recency_val;
                // Update breakdown with recency multiplier and final score
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

    // Sort by score
    all_results.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    all_results.truncate(params.limit);

    // Apply offset
    let offset = params.offset.min(all_results.len().saturating_sub(1));
    let final_results: Vec<QueryResult> = all_results.into_iter().skip(offset).collect();

    if degraded {
        Ok(SearchResponse::degraded(final_results, degraded_warning))
    } else {
        Ok(SearchResponse::new(final_results))
    }
}

/// Merge results from multiple entity types using Reciprocal Rank Fusion.
/// Partitions by type, assigns per-type ranks, then fuses.
pub fn merge_results_by_rrf(
    results: Vec<QueryResult>,
    k: f64,
    limit: usize,
) -> Vec<QueryResult> {
    // Partition by type
    let mut by_type: HashMap<String, Vec<&QueryResult>> = HashMap::new();
    for r in &results {
        by_type.entry(r.r#type.clone()).or_default().push(r);
    }

    // Compute RRF scores per ID
    let mut rrf_scores: HashMap<String, f64> = HashMap::new();
    for typed_results in by_type.values() {
        for (rank, r) in typed_results.iter().enumerate() {
            let score = 1.0 / (k + rank as f64);
            *rrf_scores.entry(r.id.clone()).or_insert(0.0) += score;
        }
    }

    // Assign RRF scores and sort
    let mut ranked: Vec<(f64, QueryResult)> = results
        .into_iter()
        .map(|r| {
            let rrf_score = rrf_scores.get(&r.id).copied().unwrap_or(0.0);
            (rrf_score, r)
        })
        .collect();
    ranked.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
    ranked.truncate(limit);
    ranked.into_iter().map(|(score, mut r)| {
        r.score = score;
        r
    }).collect()
}
