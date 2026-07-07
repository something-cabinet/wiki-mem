use petgraph::visit::EdgeRef;
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, HashSet};

const BM25_K1: f64 = 1.2;
const BM25_B: f64 = 0.75;

// ─── FSRS-6 Default Parameters ──────────────────────────────
// From open-spaced-repetition/awesome-fsrs
const FSRS_W: [f64; 21] = [
    0.212, 1.2931, 2.3065, 8.2956, 6.4133, 0.8334, 3.0194, 0.001,
    1.8722, 0.1666, 0.796, 1.4835, 0.0614, 0.2629, 1.6483, 0.6014,
    1.8729, 0.5425, 0.0912, 0.0658, 0.1542,
];

/// Compute a recency boost based on days since last update.
/// Models: "fsrs" (default), "linear", "exponential", "none".
/// `stability_days` is the half-life parameter for all models.
pub fn recency_boost(days_since_update: f64, model: &str, stability_days: f64) -> f64 {
    if days_since_update <= 0.0 {
        return 1.0;
    }
    if stability_days <= 0.0 {
        return 1.0;
    }
    match model {
        "fsrs" => {
            // FSRS-6 forgetting curve
            let w20 = FSRS_W[20];
            let factor = 0.9_f64.powf(-1.0 / w20) - 1.0;
            let r = (1.0 + factor * days_since_update / stability_days).powf(-w20);
            r.max(0.0).min(1.0)
        }
        "linear" => (1.0 - days_since_update / stability_days).max(0.0),
        "exponential" => (-days_since_update / stability_days).exp(),
        _ => 1.0, // "none" or unknown
    }
}

/// Cap total boost from multiple sources (recency × salience) to prevent domination.
pub fn cap_total_boost(recency: f64, salience: f64, max_boost: f64) -> f64 {
    (recency * salience).min(max_boost)
}

/// A single searchable document with weighted fields
#[derive(Clone, Debug)]
pub struct IndexedDoc {
    pub id: String,
    pub fields: Vec<Field>,
}

#[derive(Clone, Debug)]
pub struct Field {
    pub name: String,
    pub text: String,
    pub weight: f64,
    pub tokens: Vec<String>,
    pub term_freqs: HashMap<String, f64>,
}

impl Field {
    pub fn new(name: &str, text: &str, weight: f64) -> Self {
        let tokens = tokenize(text);
        let mut term_freqs = HashMap::new();
        for t in &tokens {
            *term_freqs.entry(t.clone()).or_insert(0.0) += 1.0;
        }
        Self {
            name: name.to_string(),
            text: text.to_string(),
            weight,
            tokens,
            term_freqs,
        }
    }
}

/// Custom BM25 index with field-weighted scoring
pub struct Bm25Index {
    pub docs: Vec<IndexedDoc>,
    pub total_docs: usize,
    pub term_freq: HashMap<String, usize>, // term → # of docs containing it
    pub field_lengths: HashMap<String, usize>, // field_name → total tokens
    pub field_doc_counts: HashMap<String, usize>, // field_name → docs with this field
}

impl Default for Bm25Index {
    fn default() -> Self {
        Self::new()
    }
}

impl Bm25Index {
    pub fn new() -> Self {
        Self {
            docs: Vec::new(),
            total_docs: 0,
            term_freq: HashMap::new(),
            field_lengths: HashMap::new(),
            field_doc_counts: HashMap::new(),
        }
    }

    pub fn build(docs: Vec<IndexedDoc>) -> Self {
        let total_docs = docs.len();
        let mut term_freq: HashMap<String, usize> = HashMap::new();
        let mut field_lengths: HashMap<String, usize> = HashMap::new();
        let mut field_doc_counts: HashMap<String, usize> = HashMap::new();

        for doc in &docs {
            let mut doc_terms: HashMap<String, bool> = HashMap::new();
            for field in &doc.fields {
                *field_lengths.entry(field.name.clone()).or_insert(0) += field.tokens.len();
                *field_doc_counts.entry(field.name.clone()).or_insert(0) += 1;
                for token in &field.tokens {
                    doc_terms.insert(token.clone(), true);
                }
            }
            for token in doc_terms.keys() {
                *term_freq.entry(token.clone()).or_insert(0) += 1;
            }
        }

        Self {
            docs,
            total_docs,
            term_freq,
            field_lengths,
            field_doc_counts,
        }
    }

    /// Score a single document against a query
    pub fn score_doc(&self, doc: &IndexedDoc, query_tokens: &[String]) -> f64 {
        let mut score = 0.0;

        for field in &doc.fields {
            let field_len = field.tokens.len();
            if field_len == 0 {
                continue;
            }

            let avg_len = self.avg_field_length(&field.name);

            for qt in query_tokens {
                let tf = field.term_freqs.get(qt).copied().unwrap_or(0.0);
                if tf == 0.0 {
                    continue;
                }

                let df = self.term_freq.get(qt).copied().unwrap_or(0) as f64;
                if df == 0.0 {
                    continue;
                }

                let idf = 1.0 + (self.total_docs as f64 - df + 0.5) / (df + 0.5);
                let denom = tf + BM25_K1 * (1.0 - BM25_B + BM25_B * (field_len as f64 / avg_len));
                let field_score = field.weight * idf * ((tf * (BM25_K1 + 1.0)) / denom);
                score += field_score;
            }
        }

        score
    }

    /// Search with BM25 + rerank boosts
    pub fn search(&self, query: &str, limit: usize) -> Vec<SearchResult> {
        let query_tokens = tokenize(query);
        if query_tokens.is_empty() || self.total_docs == 0 {
            return Vec::new();
        }

        let mut results: Vec<SearchResult> = self
            .docs
            .iter()
            .map(|doc| {
                let raw_score = self.score_doc(doc, &query_tokens);
                let boost = rerank_boost(doc, &query_tokens);
                let score = if raw_score > 0.0 {
                    raw_score + boost
                } else {
                    0.0
                };
                SearchResult {
                    id: doc.id.clone(),
                    score,
                    snippet: doc
                        .fields
                        .iter()
                        .find(|f| f.name == "title")
                        .map(|f| crate::util::truncate_str(&f.text, 120))
                        .unwrap_or_default(),
                    page_type_rank: 0,
                    centrality: 0,
                }
            })
            .filter(|r| r.score > 0.0)
            .collect();

        // Stable sort: score desc, then id alpha
        results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.id.cmp(&b.id))
        });

        // Normalize scores to 0-1 range
        normalize_scores(&mut results);

        if results.len() > limit {
            results.truncate(limit);
        }

        results
    }

    fn avg_field_length(&self, name: &str) -> f64 {
        let total = self.field_lengths.get(name).copied().unwrap_or(0) as f64;
        let count = self.field_doc_counts.get(name).copied().unwrap_or(1) as f64;
        (total / count).max(1.0)
    }
}

/// A search result with normalized score
#[derive(Clone, Debug)]
pub struct SearchResult {
    pub id: String,
    pub score: f64,
    pub snippet: String,
    pub page_type_rank: u8, // populated externally: task=7, spec=6, decision=5, concept=4, pattern=3, howto=2, reference=1
    pub centrality: usize,  // populated externally: inbound edge count
}

/// Code-aware tokenizer: preserves identifiers + sub-tokenizes on _ and -
pub fn tokenize(text: &str) -> Vec<String> {
    let lower = text.to_lowercase();
    let mut tokens = Vec::new();

    // Pass 1: extract full identifiers
    static TOKEN_RE: std::sync::LazyLock<regex::Regex> =
        std::sync::LazyLock::new(|| regex::Regex::new(r"[a-z0-9_\-]+").unwrap());
    for word in TOKEN_RE.find_iter(&lower) {
        let w = word.as_str();

        // Always add the full identifier if it has _ or -
        if w.contains('_') || w.contains('-') {
            tokens.push(w.to_string());
        }

        // Pass 2: sub-tokenize on _ and -
        for part in w.split(&['_', '-'][..]) {
            if !part.is_empty() && part.len() > 1 {
                tokens.push(part.to_string());
            }
        }
    }

    tokens
}

/// Rerank boosts: exact title match, path match, etc.
fn rerank_boost(doc: &IndexedDoc, query_tokens: &[String]) -> f64 {
    let mut boost = 0.0;
    let query_lower = query_tokens.join(" ");

    for field in &doc.fields {
        let text_lower = field.text.to_lowercase();
        if field.name == "title" {
            if text_lower == query_lower {
                boost += 8.0;
            } else if text_lower.starts_with(&query_lower) {
                boost += 4.0;
            } else if text_lower.contains(&query_lower) {
                boost += 2.0;
            }
        }
        if field.name == "id" && text_lower == query_lower {
            boost += 7.0;
        }
        if field.name == "tags" {
            for qt in query_tokens {
                if text_lower.contains(qt) {
                    boost += 3.0;
                    break;
                }
            }
        }
    }

    boost
}

/// Enrich search results with graph centrality and page type rank, then re-sort.
/// Sort order: score desc → centrality desc → page_type_rank desc → id alpha
pub fn enrich_and_sort(
    results: &mut [SearchResult],
    graph: &petgraph::stable_graph::StableGraph<
        crate::engine::WikiPageMeta,
        crate::engine::EdgeType,
    >,
    id_index: &HashMap<String, petgraph::stable_graph::NodeIndex>,
) {
    for r in results.iter_mut() {
        // Find page in graph
        if let Some(&idx) = id_index.get(&r.id) {
            let meta = &graph[idx];
            r.centrality = graph
                .edges_directed(idx, petgraph::Direction::Incoming)
                .count();
            r.page_type_rank = match meta.page_type {
                crate::engine::PageType::Task => 7,
                crate::engine::PageType::Spec => 6,
                crate::engine::PageType::Decision => 5,
                crate::engine::PageType::Concept => 4,
                crate::engine::PageType::Pattern => 3,
                crate::engine::PageType::Howto => 2,
                crate::engine::PageType::Reference => 1,
            };
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

/// Normalize scores to 0-1 range
fn normalize_scores(results: &mut [SearchResult]) {
    let max = results.iter().map(|r| r.score).fold(0.0, f64::max);
    if max <= 0.0 {
        return;
    }
    for r in results.iter_mut() {
        let n = r.score / max;
        r.score = if n > 1.0 {
            1.0
        } else if n < 0.01 && r.score > 0.0 {
            0.01
        } else {
            (n * 10000.0).round() / 10000.0
        };
    }
}

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

/// Run a unified search across pages and/or memory using the engine indexes.
/// Returns results sorted by score (or RRF-fused when both types searched),
/// with enrichment, recency boost, memory salience, and offset applied.
pub fn query(
    engine: &crate::engine::EngineState,
    params: &QueryParams,
) -> Result<Vec<QueryResult>, String> {
    use std::sync::atomic::Ordering;
    use std::sync::Arc;

    // Auto-rebuild if BM25 is empty or stale flag is set
    if engine.bm25_index.load().total_docs == 0 || engine.stale_flag.load(Ordering::Acquire) {
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
            engine.rebuild_memory_index(&memory_dir);
            engine.stale_flag.store(false, Ordering::Release);
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
        crate::embed::SearchMode::auto_detect(&params.query)
    } else {
        crate::embed::SearchMode::from_str(&params.mode)
    };

    let mut all_results: Vec<QueryResult> = Vec::new();

    // 1. Search pages
    if search_pages {
        let page_results: Vec<QueryResult> = match mode {
            crate::embed::SearchMode::Keyword => {
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
            crate::embed::SearchMode::Semantic => {
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
                    crate::embed::top_k_cosine(&query_vec.0, &vectors, params.limit);
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
            crate::embed::SearchMode::Hybrid => {
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
                        crate::embed::top_k_cosine(&query_vec.0, &vectors, params.limit * 2)
                    };

                    let fused =
                        crate::embed::rrf_fusion(&bm25_pairs, &semantic_pairs, rrf_k);
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
        let mem_index = engine.memory_index.load();
        if mem_index.total_docs > 0 {
            let mem_results = match mode {
                crate::embed::SearchMode::Keyword | crate::embed::SearchMode::Hybrid => {
                    mem_index.search(&params.query, params.limit)
                }
                _ => Vec::new(),
            };

            for r in mem_results {
                let score = r.score;
                // Salience boost: min(salience_boost, clamp / score)
                let boost = if score > 0.0 {
                    memory_salience_boost.min(memory_salience_clamp / score)
                } else {
                    1.0
                };
                all_results.push(QueryResult {
                    id: r.id,
                    score: score * boost,
                    snippet: r.snippet,
                    r#type: "memory".to_string(),
                    page_type: "memory".to_string(),
                    page_type_rank: 0,
                    centrality: 0,
                });
            }
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
    use std::collections::HashMap;

    // Partition by type
    let mut by_type: HashMap<String, Vec<&QueryResult>> = HashMap::new();
    for r in &results {
        by_type.entry(r.r#type.clone()).or_default().push(r);
    }

    // Compute RRF scores per ID
    let mut rrf_scores: HashMap<String, f64> = HashMap::new();
    for (_type, typed_results) in &by_type {
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

/// Retrieve a context pack from the wiki graph with token budget
pub fn retrieve_context(
    graph: &petgraph::stable_graph::StableGraph<
        crate::engine::WikiPageMeta,
        crate::engine::EdgeType,
    >,
    id_index: &HashMap<String, petgraph::stable_graph::NodeIndex>,
    query: &str,
    budget: usize,
    bm25_index: Option<&Bm25Index>,
) -> Vec<(String, f64, String)> {
    let budget = budget.clamp(256, 131072);
    let mut results: Vec<(String, f64, String)> = Vec::new(); // (id, score, content_slice)
    let mut tokens_used = 0usize;
    let mut visited: HashSet<String> = HashSet::new();

    // Find the match node
    let match_node = id_index
        .get(query)
        .copied()
        .or_else(|| {
            let results = match bm25_index {
                Some(idx) => idx.search(query, 1),
                None => {
                    // Rebuild BM25 from graph nodes
                    let docs: Vec<IndexedDoc> = graph
                        .node_indices()
                        .map(|idx| {
                            let meta = &graph[idx];
                            IndexedDoc {
                                id: meta.id.clone(),
                                fields: vec![
                                    Field::new("title", &meta.title, 4.0),
                                    Field::new("tags", &meta.tags.join(" "), 2.2),
                                ],
                            }
                        })
                        .collect();
                    Bm25Index::build(docs).search(query, 1)
                }
            };
            results.first().and_then(|r| id_index.get(&r.id)).copied()
        });

    let match_node = match match_node {
        Some(n) => n,
        None => return results,
    };

    let meta = &graph[match_node];
    visited.insert(meta.id.clone());

    // Add match node content with tiered truncation per token budget
    let match_text_full = format!(
        "[MATCH: {}]\nTitle: {}\n{}",
        meta.id,
        meta.title,
        meta.sources.join(", ")
    );
    let match_text_mid = format!("[MATCH: {}]\nTitle: {}", meta.id, meta.title);
    let match_text_min = format!("[MATCH: {}]", meta.id);

    let (match_text, tokens) = {
        let full_tokens = match_text_full.len() / 4;
        if tokens_used + full_tokens <= budget {
            (match_text_full, full_tokens)
        } else {
            let mid_tokens = match_text_mid.len() / 4;
            if tokens_used + mid_tokens <= budget {
                (match_text_mid, mid_tokens)
            } else {
                let min_tokens = match_text_min.len() / 4;
                if tokens_used + min_tokens <= budget {
                    (match_text_min, min_tokens)
                } else {
                    // Can't fit even tier 3 within budget — skip match node entirely
                    (String::new(), 0)
                }
            }
        }
    };
    if tokens > 0 {
        results.push((meta.id.clone(), 999.0, match_text));
        tokens_used += tokens;
    }

    // BFS: collect neighbors with edge-weighted scores
    #[derive(Clone)]
    struct ScoredNeighbor {
        node_idx: petgraph::stable_graph::NodeIndex,
        score: f64,
        edge_type: crate::engine::EdgeType,
    }

    impl PartialEq for ScoredNeighbor {
        fn eq(&self, other: &Self) -> bool {
            self.score == other.score
        }
    }
    impl Eq for ScoredNeighbor {}
    impl PartialOrd for ScoredNeighbor {
        fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
            Some(self.cmp(other))
        }
    }
    impl Ord for ScoredNeighbor {
        fn cmp(&self, other: &Self) -> Ordering {
            self.score
                .partial_cmp(&other.score)
                .unwrap_or(Ordering::Equal)
        }
    }

    let mut heap: BinaryHeap<ScoredNeighbor> = BinaryHeap::new();

    for edge in graph.edges(match_node) {
        let target = edge.target();
        let id = &graph[target].id;
        if visited.contains(id) {
            continue;
        }
        visited.insert(id.clone());

        let q_lower = query.to_lowercase();
        let title = &graph[target].title.to_lowercase();
        let relevance = if title == &q_lower {
            8.0
        } else if title.contains(&q_lower) {
            4.0
        } else {
            0.0
        };
        let score = edge.weight().priority() as f64 * (1.0 + relevance);
        heap.push(ScoredNeighbor {
            node_idx: target,
            score,
            edge_type: edge.weight().clone(),
        });
    }

    // Process neighbors in priority order, applying structural truncation
    while let Some(sn) = heap.pop() {
        if tokens_used >= budget {
            break;
        }

        let meta = &graph[sn.node_idx];
        let edge_name = format!("{:?}", sn.edge_type).to_lowercase();

        // Tier 1: full content (high relevance)
        if sn.score > 5.0 {
            let text = format!("[{}: {}]\nTitle: {}", edge_name, meta.id, meta.title);
            let tokens = text.len() / 4;
            if tokens_used + tokens <= budget {
                results.push((meta.id.clone(), sn.score, text));
                tokens_used += tokens;
            }
        }
        // Tier 2: frontmatter + headers (medium relevance)
        else if sn.score > 2.0 {
            let text = format!("[{}: {}]", edge_name, meta.id);
            let tokens = text.len() / 4;
            if tokens_used + tokens <= budget {
                results.push((meta.id.clone(), sn.score, text));
                tokens_used += tokens;
            }
        }
        // Tier 3: title + edge type (low relevance)
        else {
            let text = format!("  {} --[{}]--> {}", meta.id, edge_name, meta.title);
            let tokens = text.len() / 4;
            if tokens_used + tokens <= budget {
                results.push((meta.id.clone(), sn.score, text));
                tokens_used += tokens;
            }
        }
    }

    results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(Ordering::Equal));
    results
}

/// Rebuild a BM25 index from memory JSON files in a directory.
/// Returns the built index and the number of entries indexed.
pub fn rebuild_memory_index_from_dir(memory_dir: &std::path::Path) -> (Bm25Index, usize) {
    use std::fs;

    if !memory_dir.exists() {
        return (Bm25Index::new(), 0);
    }

    let mut docs = Vec::new();
    for entry in walkdir::WalkDir::new(memory_dir).follow_links(false).into_iter().filter_map(|e| {
        if let Err(err) = &e {
            tracing::warn!("Memory dir walk error: {}", err);
        }
        e.ok()
    }) {
        if !entry.file_type().is_file() || entry.path().extension().map(|e| e != "json").unwrap_or(true) {
            continue;
        }
        let content = match fs::read_to_string(entry.path()) {
            Ok(c) => c,
            Err(_) => continue,
        };
        if let Ok(mem) = serde_json::from_str::<crate::engine::MemoryEntry>(&content) {
            let doc_id = format!("memory:{}", mem.id);
            docs.push(IndexedDoc {
                id: doc_id,
                fields: vec![
                    Field::new("title", &mem.title, 4.0),
                    Field::new("body", &mem.content, 1.0),
                ],
            });
        }
    }

    let index = Bm25Index::build(docs);
    let count = index.total_docs;
    (index, count)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_index() -> Bm25Index {
        let docs = vec![
            IndexedDoc {
                id: "wiki:concepts:auth".into(),
                fields: vec![
                    Field::new("title", "Authentication Architecture", 4.0),
                    Field::new("tags", "auth security oauth2", 2.2),
                    Field::new("body", "JWT tokens with RS256 signing", 1.0),
                ],
            },
            IndexedDoc {
                id: "wiki:patterns:oauth2".into(),
                fields: vec![
                    Field::new("title", "OAuth2 Authorization Flow", 4.0),
                    Field::new("tags", "auth oauth2 security", 2.2),
                    Field::new("body", "Authorization code grant with PKCE extension", 1.0),
                ],
            },
            IndexedDoc {
                id: "wiki:reference:errors".into(),
                fields: vec![
                    Field::new("title", "Error Codes Reference", 4.0),
                    Field::new("tags", "errors reference", 2.2),
                    Field::new(
                        "body",
                        "ERR_AUTH_401: token expired, ERR_AUTH_403: forbidden",
                        1.0,
                    ),
                ],
            },
        ];
        Bm25Index::build(docs)
    }

    #[test]
    fn test_field_weighted_scoring() {
        let index = make_test_index();
        let results = index.search("authentication", 10);
        assert!(!results.is_empty());
        // Authentication should rank highest
        assert_eq!(results[0].id, "wiki:concepts:auth");
    }

    #[test]
    fn test_code_aware_tokenizer() {
        let tokens = tokenize("ERR_AUTH_401");
        assert!(tokens.contains(&"err_auth_401".to_string()));
        assert!(tokens.contains(&"err".to_string()));
        assert!(tokens.contains(&"auth".to_string()));
        assert!(tokens.contains(&"401".to_string()));
    }

    #[test]
    fn test_search_finds_error_code() {
        let index = make_test_index();
        let results = index.search("ERR_AUTH_401", 10);
        assert!(!results.is_empty());
        assert_eq!(results[0].id, "wiki:reference:errors");
    }

    #[test]
    fn test_score_normalization() {
        let index = make_test_index();
        let results = index.search("oauth2", 10);
        assert!(!results.is_empty(), "oauth2 should match the OAuth2 page");
        for r in &results {
            assert!(
                r.score >= 0.0 && r.score <= 1.0,
                "Score {} out of range for {}",
                r.score,
                r.id
            );
        }
    }

    #[test]
    fn test_tokenize_empty() {
        let tokens = tokenize("");
        assert!(tokens.is_empty());
    }

    #[test]
    fn test_tokenize_kebab_case() {
        let tokens = tokenize("auth-service");
        assert!(tokens.contains(&"auth-service".to_string()));
        assert!(tokens.contains(&"auth".to_string()));
        assert!(tokens.contains(&"service".to_string()));
    }

    #[test]
    fn test_zero_result_guard() {
        let index = make_test_index();
        // Gibberish query should return no results
        let results = index.search("xyznonexistent123!!!", 10);
        assert!(results.is_empty());
    }

    #[test]
    fn test_recency_boost_fsrs_day0() {
        let b = recency_boost(0.0, "fsrs", 7.0);
        assert!((b - 1.0).abs() < 1e-6, "Day 0 should be 1.0, got {b}");
    }

    #[test]
    fn test_recency_boost_fsrs_day7() {
        let b = recency_boost(7.0, "fsrs", 7.0);
        assert!((b - 0.9).abs() < 0.01, "Day 7 (t=S) should be ~0.9, got {b}");
    }

    #[test]
    fn test_recency_boost_fsrs_day30() {
        let b = recency_boost(30.0, "fsrs", 7.0);
        assert!(b > 0.6 && b < 0.9, "Day 30 S=7 should be ~0.78, got {b}");
    }

    #[test]
    fn test_recency_boost_linear() {
        assert!((recency_boost(0.0, "linear", 7.0) - 1.0).abs() < 1e-6);
        assert!((recency_boost(7.0, "linear", 7.0) - 0.0).abs() < 1e-6);
        assert!((recency_boost(3.5, "linear", 7.0) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_recency_boost_exponential() {
        assert!((recency_boost(0.0, "exponential", 7.0) - 1.0).abs() < 1e-6);
        let b = recency_boost(7.0, "exponential", 7.0);
        assert!((b - 0.3679).abs() < 0.01, "Day 7 should be ~0.368, got {b}");
    }

    #[test]
    fn test_recency_boost_none() {
        assert!((recency_boost(0.0, "none", 7.0) - 1.0).abs() < 1e-6);
        assert!((recency_boost(100.0, "none", 7.0) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_recency_boost_zero_stability() {
        assert!((recency_boost(5.0, "fsrs", 0.0) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_cap_total_boost() {
        assert!((cap_total_boost(1.0, 1.0, 4.0) - 1.0).abs() < 1e-6);
        assert!((cap_total_boost(3.0, 2.0, 4.0) - 4.0).abs() < 1e-6);
        assert!((cap_total_boost(1.0, 1.0, 1.0) - 1.0).abs() < 1e-6);
    }
}
