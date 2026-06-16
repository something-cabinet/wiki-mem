use petgraph::visit::EdgeRef;
use std::collections::{HashMap, HashSet, BinaryHeap};
use std::cmp::Ordering;

const BM25_K1: f64 = 1.2;
const BM25_B: f64 = 0.75;

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
}

impl Field {
    pub fn new(name: &str, text: &str, weight: f64) -> Self {
        Self {
            name: name.to_string(),
            text: text.to_string(),
            weight,
            tokens: tokenize(text),
        }
    }
}

/// Custom BM25 index with field-weighted scoring
pub struct Bm25Index {
    pub docs: Vec<IndexedDoc>,
    pub total_docs: usize,
    pub term_freq: HashMap<String, usize>,         // term → # of docs containing it
    pub field_lengths: HashMap<String, usize>,      // field_name → total tokens
    pub field_doc_counts: HashMap<String, usize>,   // field_name → docs with this field
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

        Self { docs, total_docs, term_freq, field_lengths, field_doc_counts }
    }

    /// Score a single document against a query
    pub fn score_doc(&self, doc: &IndexedDoc, query_tokens: &[String]) -> f64 {
        let mut score = 0.0;

        for field in &doc.fields {
            let field_len = field.tokens.len();
            if field_len == 0 { continue; }

            let avg_len = self.avg_field_length(&field.name);

            for qt in query_tokens {
                let tf = field.tokens.iter().filter(|t| *t == qt).count() as f64;
                if tf == 0.0 { continue; }

                let df = self.term_freq.get(qt).copied().unwrap_or(0) as f64;
                if df == 0.0 { continue; }

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

        let mut results: Vec<SearchResult> = self.docs.iter().map(|doc| {
            let raw_score = self.score_doc(doc, &query_tokens);
            let boost = rerank_boost(doc, &query_tokens);
            let score = if raw_score > 0.0 { raw_score + boost } else { 0.0 };
            SearchResult {
                id: doc.id.clone(),
                score,
                snippet: doc.fields.iter()
                    .find(|f| f.name == "title")
                    .map(|f| truncate(&f.text, 120))
                    .unwrap_or_default(),
            }
        }).filter(|r| r.score > 0.0).collect();

        // Stable sort: score desc, then id alpha
        results.sort_by(|a, b| {
            b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal)
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
}

/// Code-aware tokenizer: preserves identifiers + sub-tokenizes on _ and -
pub fn tokenize(text: &str) -> Vec<String> {
    let lower = text.to_lowercase();
    let mut tokens = Vec::new();

    // Pass 1: extract full identifiers
    let re = regex::Regex::new(r"[a-z0-9_\-]+").unwrap();
    for word in re.find_iter(&lower) {
        let w = word.as_str().to_string();

        // Always add the full identifier if it has _ or -
        if w.contains('_') || w.contains('-') {
            tokens.push(w.clone());
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
            if text_lower == query_lower { boost += 8.0; }
            else if text_lower.starts_with(&query_lower) { boost += 4.0; }
            else if text_lower.contains(&query_lower) { boost += 2.0; }
        }
        if field.name == "id" && text_lower == query_lower {
            boost += 7.0;
        }
        if field.name == "tags" {
            for qt in query_tokens {
                if text_lower.contains(qt) { boost += 3.0; break; }
            }
        }
    }

    boost
}

/// Normalize scores to 0-1 range
fn normalize_scores(results: &mut [SearchResult]) {
    let max = results.iter().map(|r| r.score).fold(0.0, f64::max);
    if max <= 0.0 { return; }
    for r in results.iter_mut() {
        let n = r.score / max;
        r.score = if n > 1.0 { 1.0 }
            else if n < 0.01 && r.score > 0.0 { 0.01 }
            else { (n * 10000.0).round() / 10000.0 };
    }
}

/// Retrieve a context pack from the wiki graph with token budget
pub fn retrieve_context(
    graph: &petgraph::stable_graph::StableGraph<crate::engine::WikiPageMeta, crate::engine::EdgeType>,
    id_index: &HashMap<String, petgraph::stable_graph::NodeIndex>,
    query: &str,
    budget: usize,
) -> Vec<(String, f64, String)> {
    let query_tokens = tokenize(query);
    let budget = budget.clamp(256, 131072);
    let mut results: Vec<(String, f64, String)> = Vec::new(); // (id, score, content_slice)
    let mut tokens_used = 0usize;
    let mut visited: HashSet<String> = HashSet::new();

    // Find the match node
    let match_node = id_index.get(query).or_else(|| {
        // Try BM25 search to find the best match
        let docs: Vec<IndexedDoc> = graph.node_indices().map(|idx| {
            let meta = &graph[idx];
            IndexedDoc {
                id: meta.id.clone(),
                fields: vec![
                    Field::new("title", &meta.title, 4.0),
                    Field::new("tags", &meta.tags.join(" "), 2.2),
                ],
            }
        }).collect();
        let bm25 = Bm25Index::build(docs);
        bm25.search(query, 1).first().and_then(|r| id_index.get(&r.id))
    }).copied();

    let match_node = match match_node {
        Some(n) => n,
        None => return results,
    };

    let meta = &graph[match_node];
    visited.insert(meta.id.clone());

    // Add match node content (tier 1: full)
    let match_text = format!("[MATCH: {}]\nTitle: {}", meta.id, meta.title);
    let tokens = match_text.len() / 4;
    if tokens_used + tokens <= budget || results.is_empty() {
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
            self.score.partial_cmp(&other.score)
        }
    }
    impl Ord for ScoredNeighbor {
        fn cmp(&self, other: &Self) -> Ordering {
            self.score.partial_cmp(&other.score).unwrap_or(Ordering::Equal)
        }
    }

    let mut heap: BinaryHeap<ScoredNeighbor> = BinaryHeap::new();

    for edge in graph.edges(match_node) {
        let target = edge.target();
        let id = &graph[target].id;
        if visited.contains(id) { continue; }
        visited.insert(id.clone());

        let bm25_score = {
            let docs = vec![IndexedDoc {
                id: id.clone(),
                fields: vec![Field::new("title", &graph[target].title, 4.0)],
            }];
            let bm25 = Bm25Index::build(docs);
            bm25.search(query, 1).first().map(|r| r.score).unwrap_or(0.0)
        };
        let score = edge.weight().priority() as f64 * (1.0 + bm25_score);
        heap.push(ScoredNeighbor { node_idx: target, score, edge_type: *edge.weight() });
    }

    // Process neighbors in priority order, applying structural truncation
    while let Some(sn) = heap.pop() {
        if tokens_used >= budget { break; }

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

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max { s.to_string() }
    else { format!("{}...", &s[..max.saturating_sub(3)]) }
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
                    Field::new("body", "ERR_AUTH_401: token expired, ERR_AUTH_403: forbidden", 1.0),
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
            assert!(r.score >= 0.0 && r.score <= 1.0,
                    "Score {} out of range for {}", r.score, r.id);
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
}
