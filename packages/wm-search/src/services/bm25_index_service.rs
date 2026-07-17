// BM25 search engine with field-weighted scoring.

//! BM25 search index — field-weighted token-based search engine with
//! code-aware tokenization, rerank boosts, and parallel build.

use rayon::prelude::*;
use std::collections::{HashMap, HashSet};

use super::field_model::tokenize;
use super::indexed_doc_model::IndexedDoc;
use super::search_result_model::SearchResult;
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
                        .map(|f| wm_util::truncate_str(&f.text, 120))
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
