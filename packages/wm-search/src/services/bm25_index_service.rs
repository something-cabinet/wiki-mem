// BM25 search engine with field-weighted scoring.

//! BM25 search index — field-weighted token-based search engine with
//! code-aware tokenization, rerank boosts, and parallel build.

use rayon::prelude::*;
use std::collections::{HashMap, HashSet};

use super::field_model::tokenize;
use super::indexed_doc_model::IndexedDoc;
use super::search_result_model::{ScoreBreakdown, SearchResult};
use crate::helpers::scoring_helper::{BM25_K1, BM25_B};

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

    /// Add a single document to the index incrementally.
    /// Tokenizes the doc, updates term frequencies, field lengths, field doc counts, and total_docs.
    pub fn add_document(&mut self, doc: IndexedDoc) {
        // Collect unique terms across all fields (same as build())
        let mut doc_terms: HashSet<String> = HashSet::new();

        for field in &doc.fields {
            *self.field_lengths.entry(field.name.clone()).or_insert(0) += field.tokens.len();
            *self.field_doc_counts.entry(field.name.clone()).or_insert(0) += 1;
            for token in &field.tokens {
                doc_terms.insert(token.clone());
            }
        }

        for token in doc_terms {
            *self.term_freq.entry(token).or_insert(0) += 1;
        }

        self.docs.push(doc);
        self.total_docs += 1;
    }

    /// Remove a document from the index by its ID.
    /// Re-tokenizes the document content to know which terms to decrement.
    pub fn remove_document(&mut self, doc_id: &str) {
        let pos = match self.docs.iter().position(|d| d.id == doc_id) {
            Some(p) => p,
            None => return,
        };

        let doc = self.docs.swap_remove(pos);

        // Collect unique terms across all fields (same structure as build/add)
        let mut doc_terms: HashSet<String> = HashSet::new();

        for field in &doc.fields {
            let len = field.tokens.len();
            if let Some(v) = self.field_lengths.get_mut(&field.name) {
                *v = v.saturating_sub(len);
                if *v == 0 {
                    self.field_lengths.remove(&field.name);
                }
            }

            if let Some(v) = self.field_doc_counts.get_mut(&field.name) {
                *v = v.saturating_sub(1);
                if *v == 0 {
                    self.field_doc_counts.remove(&field.name);
                }
            }

            for token in &field.tokens {
                doc_terms.insert(token.clone());
            }
        }

        for token in doc_terms {
            if let Some(v) = self.term_freq.get_mut(&token) {
                *v = v.saturating_sub(1);
            }
        }

        self.total_docs = self.total_docs.saturating_sub(1);
    }

    /// Replace a document in the index (remove old + add new).
    pub fn update_document(&mut self, doc_id: &str, new_doc: IndexedDoc) {
        self.remove_document(doc_id);
        self.add_document(new_doc);
    }

    /// Get the number of documents in the index.
    pub fn doc_count(&self) -> usize {
        self.total_docs
    }

    pub fn build(docs: Vec<IndexedDoc>) -> Self {
        let total_docs = docs.len();
        let mut term_freq: HashMap<String, usize> = HashMap::new();
        let mut field_lengths: HashMap<String, usize> = HashMap::new();
        let mut field_doc_counts: HashMap<String, usize> = HashMap::new();

        // Parallel per-doc: collect per-doc partials, then sequential merge
        #[derive(Default)]
        struct DocPartial {
            field_lengths: HashMap<String, usize>,
            field_doc_counts: HashMap<String, usize>,
            doc_terms: HashSet<String>,
        }

        let partials: Vec<DocPartial> = docs
            .par_iter()
            .map(|doc| {
                let mut p = DocPartial::default();
                for field in &doc.fields {
                    *p.field_lengths.entry(field.name.clone()).or_insert(0) += field.tokens.len();
                    *p.field_doc_counts.entry(field.name.clone()).or_insert(0) += 1;
                    for token in &field.tokens {
                        p.doc_terms.insert(token.clone());
                    }
                }
                p
            })
            .collect();

        for partial in partials {
            for (field, len) in partial.field_lengths {
                *field_lengths.entry(field).or_insert(0) += len;
            }
            for (field, count) in partial.field_doc_counts {
                *field_doc_counts.entry(field).or_insert(0) += count;
            }
            for token in partial.doc_terms {
                *term_freq.entry(token).or_insert(0) += 1;
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
            .par_iter()
            .map(|doc| {
                let raw_score = self.score_doc(doc, &query_tokens);
                let score = if raw_score > 0.0 {
                    raw_score
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
                        .map(|f| crate::helpers::truncate_str(&f.text, 120))
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

/// Rerank boosts: exact title match, path match, etc.
pub fn rerank_boost(doc: &IndexedDoc, query_tokens: &[String]) -> f64 {
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

/// Apply rerank boosts to search results after RRF fusion.
/// Knowns-inspired: title density, exact match, tag overlap.
///
/// These boosts use small additive values designed for the post-normalization
/// score range (~0–1), unlike the old pre-normalization boosts (+8, +7, +3).
///
/// Returns a map of doc ID → score breakdown showing each bonus contribution.
/// Callers should fill in `bm25`, `semantic`, `recency`, and `final_score`
/// after this function returns.
pub fn post_rrf_rerank(
    results: &mut [(String, f64)],
    docs: &[IndexedDoc],
    query_tokens: &[String],
) -> HashMap<String, ScoreBreakdown> {
    let query_lower = query_tokens.join(" ");
    let query_word_count = query_tokens.len() as f64;

    // Capture the pre-rerank RRF score for each result
    let pre_rrf: HashMap<String, f64> = results.iter().map(|(id, s)| (id.clone(), *s)).collect();
    let mut breakdowns: HashMap<String, ScoreBreakdown> = HashMap::new();

    for (id, score) in results.iter_mut() {
        let mut bd = ScoreBreakdown {
            bm25: 0.0,
            rrf: pre_rrf.get(id).copied().unwrap_or(0.0),
            semantic: 0.0,
            title_density: 0.0,
            exact_title: 0.0,
            tag_overlap: 0.0,
            exact_id: 0.0,
            recency: 0.0,
            final_score: 0.0,
        };

        if let Some(doc) = docs.iter().find(|d| d.id == *id) {
            for field in &doc.fields {
                let text_lower = field.text.to_lowercase();

                match field.name.as_str() {
                    "title" => {
                        // Title density: +0.03 per query word found in title
                        let matched = query_tokens
                            .iter()
                            .filter(|qt| text_lower.contains(qt.as_str()))
                            .count() as f64;
                        let td = matched * 0.03;
                        *score += td;
                        bd.title_density = td;

                        // Exact title match: +0.15
                        if text_lower == query_lower {
                            *score += 0.15;
                            bd.exact_title = 0.15;
                        }
                    }
                    "tags" => {
                        // Proportional tag overlap (uses current score which includes title bonuses)
                        let matched = query_tokens
                            .iter()
                            .filter(|qt| text_lower.contains(qt.as_str()))
                            .count() as f64;
                        if matched > 0.0 {
                            let overlap = (matched / query_word_count) * 0.1 * *score;
                            *score += overlap;
                            bd.tag_overlap = overlap;
                        }
                    }
                    "id" => {
                        // Exact ID match: +0.10
                        if text_lower == query_lower {
                            *score += 0.10;
                            bd.exact_id = 0.10;
                        }
                    }
                    _ => {}
                }
            }
        }

        breakdowns.insert(id.clone(), bd);
    }

    breakdowns
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::field_model::Field;

    #[test]
    fn test_bm25_add_document() {
        let mut index = Bm25Index::new();
        let doc = IndexedDoc {
            id: "test-doc-1".to_string(),
            fields: vec![
                Field::new("title", "Hello World", 1.0),
                Field::new("content", "This is some test content for BM25", 0.5),
            ],
        };
        index.add_document(doc);
        assert_eq!(index.doc_count(), 1);
    }

    #[test]
    fn test_bm25_remove_document() {
        let mut index = Bm25Index::new();
        let doc = IndexedDoc {
            id: "test-id".to_string(),
            fields: vec![Field::new("title", "Remove me", 1.0)],
        };
        index.add_document(doc);
        assert_eq!(index.doc_count(), 1);

        index.remove_document("test-id");
        assert_eq!(index.doc_count(), 0);
    }

    #[test]
    fn test_bm25_update_document() {
        let mut index = Bm25Index::new();

        let old_doc = IndexedDoc {
            id: "test-id".to_string(),
            fields: vec![
                Field::new("title", "Old Title", 1.0),
                Field::new("content", "obsolete deprecated text", 0.5),
            ],
        };
        index.add_document(old_doc);
        assert_eq!(index.doc_count(), 1);

        let new_doc = IndexedDoc {
            id: "test-id".to_string(),
            fields: vec![
                Field::new("title", "New Title", 1.0),
                Field::new("content", "brand new fresh material", 0.5),
            ],
        };
        index.update_document("test-id", new_doc);
        assert_eq!(index.doc_count(), 1);

        // Search for old content — should have no results
        let results = index.search("obsolete deprecated text", 10);
        assert!(
            results.is_empty(),
            "should not find old content after update"
        );

        // Search for new content — should have results
        let results = index.search("brand new fresh", 10);
        assert!(!results.is_empty(), "should find new content after update");
    }

    #[test]
    fn test_bm25_add_then_search() {
        let mut index = Bm25Index::new();
        let doc = IndexedDoc {
            id: "searchable-doc".to_string(),
            fields: vec![
                Field::new("title", "Unique Term Doc", 1.0),
                Field::new("content", "this document contains unique_search_term_xyz for testing", 0.5),
            ],
        };
        index.add_document(doc);

        let results = index.search("unique_search_term_xyz", 10);
        assert!(!results.is_empty(), "should find the document by unique term");
    }
}
