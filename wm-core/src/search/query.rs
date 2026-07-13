//! Unified search query — orchestrates BM25, semantic, and hybrid searches
//! across wiki pages and memory entries, with RRF fusion and enrichment.

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::Ordering as AtomicOrdering;

use petgraph::Direction;

use super::index::{Bm25Index, Field, IndexedDoc, SearchResult};
use super::scoring::recency_boost;
use crate::embed::{rrf_fusion, top_k_cosine, SearchMode};
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
                .count();
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
pub fn run_unified_search(
    engine: &EngineState,
    params: &QueryParams,
) -> Result<Vec<QueryResult>, String> {
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
                    ],
                })
                .collect();
            engine.bm25_index.store(Arc::new(Bm25Index::build(docs)));
            let memory_dir = root.join(".wm").join("memory");
            engine.rebuild_memory_index_from_disk(&memory_dir);
            engine.stale_flag.store(false, AtomicOrdering::Release);
        }
    }

    let embedder_loaded = engine.embedder.is_loaded();

    // Snapshot for enrichment
    let snap = engine.graph.load();
    let graph = &snap.0;
    let id_index = &snap.1;

    let search_pages = params.r#type == "all" || params.r#type == "page" || params.r#type == "task";
    let search_memory = params.r#type == "all" || params.r#type == "memory";

    // Acquire config values
    let config_guard = engine
        .config
        .read()
        .map_err(|e| format!("config lock poisoned: {}", e))?;
    let rrf_k = config_guard.search.rrf_k as f64;
    let recency_model = config_guard.search.scoring.recency_model.clone();
    let recency_stability = config_guard.search.scoring.recency_stability_days as f64;
    let memory_salience_boost = config_guard.search.scoring.memory_salience_boost;
    let memory_salience_clamp = config_guard.search.scoring.memory_salience_clamp;
    drop(config_guard);

    // Determine search mode
    let mode = if params.mode == "auto" {
        SearchMode::auto_detect(&params.query)
    } else {
        SearchMode::from_str(&params.mode)
    };

    let mut all_results: Vec<QueryResult> = Vec::new();

    // 1. Search pages
    if search_pages {
        let page_results: Vec<QueryResult> = match mode {
            SearchMode::Keyword => {
                let bm25 = engine.bm25_index.load();
                let r = bm25.search(&params.query, params.limit);
                r.iter()
                    .map(|r| QueryResult {
                        id: r.id.clone(),
                        score: r.score,
                        snippet: r.snippet.clone(),
                        r#type: "page".to_string(),
                        page_type: String::new(),
                        page_type_rank: r.page_type_rank,
                        centrality: r.centrality,
                    })
                    .collect()
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
                top_k
                    .into_iter()
                    .map(|(id, score)| QueryResult {
                        id,
                        score,
                        snippet: String::new(),
                        r#type: "page".to_string(),
                        page_type: String::new(),
                        page_type_rank: 0,
                        centrality: 0,
                    })
                    .collect()
            }
            SearchMode::Hybrid => {
                if !embedder_loaded {
                    let bm25 = engine.bm25_index.load();
                    let r = bm25.search(&params.query, params.limit);
                    r.iter()
                        .map(|r| QueryResult {
                            id: r.id.clone(),
                            score: r.score,
                            snippet: r.snippet.clone(),
                            r#type: "page".to_string(),
                            page_type: String::new(),
                            page_type_rank: r.page_type_rank,
                            centrality: r.centrality,
                        })
                        .collect()
                } else {
                    let bm25 = engine.bm25_index.load();
                    let bm25_results = bm25.search(&params.query, params.limit * 2);
                    let bm25_pairs: Vec<(String, f64)> = bm25_results
                        .iter()
                        .map(|r| (r.id.clone(), r.score))
                        .collect();

                    let vectors = engine.vector_store.snapshot();
                    let query_vec = engine
                        .embedder
                        .embed(&params.query)
                        .map_err(|e| format!("Embedding failed: {}", e))?;
                    let semantic_pairs = if vectors.is_empty() {
                        Vec::new()
                    } else {
                        top_k_cosine(&query_vec.0, &vectors, params.limit * 2)
                    };

                    let fused =
                        rrf_fusion(&bm25_pairs, &semantic_pairs, rrf_k);
                    let truncated: Vec<_> = fused.into_iter().take(params.limit).collect();
                    truncated
                        .into_iter()
                        .map(|(id, score)| QueryResult {
                            id,
                            score,
                            snippet: String::new(),
                            r#type: "page".to_string(),
                            page_type: String::new(),
                            page_type_rank: 0,
                            centrality: 0,
                        })
                        .collect()
                }
            }
        };

        // Enrich with page type info and apply recency boost
        for mut r in page_results {
            let id = r.id.clone();

            // Enrich from graph
            if let Some(&idx) = id_index.get(&id) {
                let meta = &graph[idx];
                r.page_type = format!("{:?}", meta.page_type).to_lowercase();
                r.centrality = meta.relates_to.len();
                r.page_type_rank = meta.page_type.priority_rank();
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
                let recency =
                    recency_boost(days_since, &recency_model, recency_stability);
                r.score *= recency;
            }

            all_results.push(r);
        }
    }

    // 2. Search memory
    if search_memory {
        // 2a. BM25 keyword search
        let mem_index = engine.memory_index.load();
        let mem_results = if mem_index.total_docs > 0 {
            match mode {
                SearchMode::Keyword | SearchMode::Hybrid => {
                    mem_index.search(&params.query, params.limit)
                }
                _ => Vec::new(),
            }
        } else {
            Vec::new()
        };

        // 2b. Semantic vector search for memory (if embedder loaded and vectors exist)
        let mem_vec_results: Vec<(String, f64)> = if mode == SearchMode::Semantic || mode == SearchMode::Hybrid {
            let mem_vectors = engine.memory_vectors.load();
            if embedder_loaded && !mem_vectors.is_empty() {
                match engine.embedder.embed(&params.query) {
                    Ok(query_vec) => {
                        top_k_cosine(&query_vec.0, &mem_vectors, params.limit)
                    }
                    Err(_) => Vec::new(),
                }
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        };

        // Merge keyword + semantic memory results (simple score merge, no RRF between memory modes)
        let all_mem_results: Vec<(String, f64)> = {
            let mut merged: HashMap<String, f64> = HashMap::new();
            for r in &mem_results {
                merged.insert(r.id.clone(), r.score);
            }
            for (id, score) in &mem_vec_results {
                let entry = merged.entry(id.clone()).or_insert(0.0);
                if *score > *entry {
                    *entry = *score;
                }
            }
            let mut list: Vec<_> = merged.into_iter().collect();
            list.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
            list.truncate(params.limit);
            list
        };

        for (mem_id, score) in all_mem_results {
            let boost = if score > 0.0 {
                memory_salience_boost.min(memory_salience_clamp / score)
            } else {
                1.0
            };
            all_results.push(QueryResult {
                id: mem_id,
                score: score * boost,
                snippet: String::new(),
                r#type: "memory".to_string(),
                page_type: "memory".to_string(),
                page_type_rank: 0,
                centrality: 0,
            });
        }
    }

    // Merge / sort
    let merged = if search_pages && search_memory && all_results.len() > 1 {
        merge_results_by_rrf(all_results, rrf_k, params.limit)
    } else {
        all_results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        all_results.truncate(params.limit);
        all_results
    };

    // Apply offset
    let offset = params.offset.min(merged.len().saturating_sub(1));
    Ok(merged.into_iter().skip(offset).collect())
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
