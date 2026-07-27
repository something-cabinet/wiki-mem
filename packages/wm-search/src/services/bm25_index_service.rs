// BM25 search engine with field-weighted scoring.

//! BM25 search index — field-weighted token-based search engine with
//! code-aware tokenization, rerank boosts, and parallel build.

use rayon::prelude::*;
use std::collections::{HashMap, HashSet};

use super::field_model::{stem_word, tokenize};
use super::indexed_doc_model::IndexedDoc;
use super::search_result_model::{ScoreBreakdown, SearchResult};
use crate::helpers::scoring_helper::{BM25_B, BM25_K1};

/// Convert a `usize` to `f64` without triggering `clippy::as_conversions`.
///
/// `From<usize> for f64` does not exist in std because `f64` cannot represent
/// every `usize` value on 64-bit platforms. This uses a `u32` intermediate
/// since `From<u32> for f64` is available. For our use-case (document counts,
/// field lengths, query token counts) the values are well within `u32` range.
fn usize_to_f64(v: usize) -> f64 {
    f64::from(u32::try_from(v).expect("usize value exceeds u32 range"))
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

    /// Add a single document to the index incrementally.
    /// Tokenizes the doc, updates term frequencies, field lengths, field doc counts, and total_docs.
    pub fn add_document(&mut self, doc: IndexedDoc) {
        // Collect unique terms across all fields (same as build())
        let mut doc_terms: HashSet<String> = HashSet::new();

        for field in &doc.fields {
            let len = field.tokens.len();
            self.field_lengths
                .entry(field.name.clone())
                .and_modify(|v| *v = v.wrapping_add(len))
                .or_insert(len);
            self.field_doc_counts
                .entry(field.name.clone())
                .and_modify(|v| *v = v.wrapping_add(1))
                .or_insert(1);
            for token in &field.tokens {
                doc_terms.insert(token.clone());
            }
        }

        for token in doc_terms {
            self.term_freq
                .entry(token)
                .and_modify(|v| *v = v.wrapping_add(1))
                .or_insert(1);
        }

        self.docs.push(doc);
        self.total_docs = self.total_docs.wrapping_add(1);
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
                    let len = field.tokens.len();
                    p.field_lengths
                        .entry(field.name.clone())
                        .and_modify(|v| *v = v.wrapping_add(len))
                        .or_insert(len);
                    p.field_doc_counts
                        .entry(field.name.clone())
                        .and_modify(|v| *v = v.wrapping_add(1))
                        .or_insert(1);
                    for token in &field.tokens {
                        p.doc_terms.insert(token.clone());
                    }
                }
                p
            })
            .collect();

        for partial in partials {
            for (field, len) in partial.field_lengths {
                field_lengths
                    .entry(field)
                    .and_modify(|v| *v = v.wrapping_add(len))
                    .or_insert(len);
            }
            for (field, count) in partial.field_doc_counts {
                field_doc_counts
                    .entry(field)
                    .and_modify(|v| *v = v.wrapping_add(count))
                    .or_insert(count);
            }
            for token in partial.doc_terms {
                term_freq
                    .entry(token)
                    .and_modify(|v| *v = v.wrapping_add(1))
                    .or_insert(1);
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

                let df = usize_to_f64(self.term_freq.get(qt).copied().unwrap_or(0));
                if df == 0.0 {
                    continue;
                }

                // Standard BM25 IDF (Robertson-Sparck Jones with ln smoothing)
                let total_docs_f = usize_to_f64(self.total_docs);
                let idf = (1.0 + (total_docs_f - df + 0.5) / (df + 0.5)).ln();
                let field_len_f = usize_to_f64(field_len);
                let denom = tf + BM25_K1 * (1.0 - BM25_B + BM25_B * (field_len_f / avg_len));
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
        let query_lower = query.to_lowercase();

        let mut results: Vec<SearchResult> = self
            .docs
            .par_iter()
            .map(|doc| {
                let raw_score = self.score_doc(doc, &query_tokens);
                let rerank = rerank_boost(doc, &query_lower, &query_tokens);
                let score = if raw_score > 0.0 || rerank > 0.0 {
                    raw_score + rerank
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
        let total = usize_to_f64(self.field_lengths.get(name).copied().unwrap_or(0));
        let count = usize_to_f64(self.field_doc_counts.get(name).copied().unwrap_or(1));
        (total / count).max(1.0)
    }
}

/// Rerank boosts: exact title match, path match, etc.
///
/// Uses `query_lower` (the lowercased query string) for phrase-level
/// checks (exact/starts-with/contains) so stemming doesn't break matching.
/// Uses `query_tokens` (with stemmed variants) only for per-token checks.
/// Caller should pass `query_lower` already lowered (pre-hoisted).
pub fn rerank_boost(doc: &IndexedDoc, query_lower: &str, query_tokens: &[String]) -> f64 {
    let mut boost = 0.0;

    for field in &doc.fields {
        match field.name.as_str() {
            "title" => {
                let text_lower = field.text.to_lowercase();
                if text_lower == query_lower {
                    boost += 8.0;
                } else if stem_word(&text_lower) == stem_word(query_lower) {
                    // Stemmed forms match: "design patterns" ↔ "design pattern", "styling" ↔ "style"
                    boost += 8.0;
                } else if text_lower.starts_with(query_lower) {
                    // Title starts with query: "design patterns" ← "design pattern"
                    boost += 4.0;
                } else if query_lower.starts_with(&text_lower) {
                    // Query starts with title: "design patterns" → "design pattern"
                    boost += 4.0;
                } else if text_lower.contains(query_lower) {
                    boost += 2.0;
                }
            }
            "id" if field.text.to_lowercase() == query_lower => {
                boost += 7.0;
            }
            "tags" => {
                let text_lower = field.text.to_lowercase();
                for qt in query_tokens {
                    if text_lower.contains(qt) {
                        boost += 3.0;
                        break;
                    }
                }
            }
            _ => {}
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
    query_raw: &str,
    query_tokens: &[String],
) -> HashMap<String, ScoreBreakdown> {
    let query_lower = query_raw.to_lowercase();
    let query_word_count = usize_to_f64(query_tokens.len());

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
            title_starts_with: 0.0,
            title_contains: 0.0,
            tag_overlap: 0.0,
            exact_id: 0.0,
            recency: 0.0,
            final_score: 0.0,
        };

        if let Some(doc) = docs.iter().find(|d| d.id == *id) {
            for field in &doc.fields {
                match field.name.as_str() {
                    "title" => {
                        let text_lower = field.text.to_lowercase();
                        // Title density: +0.03 per query word found in title
                        let matched = usize_to_f64(
                            query_tokens
                                .iter()
                                .filter(|qt| text_lower.contains(qt.as_str()))
                                .count(),
                        );
                        let td = matched * 0.03;
                        *score += td;
                        bd.title_density = td;

                        // Exact title match: +0.15 (uses raw query first, then stemmed for variants)
                        if text_lower == query_lower || stem_word(&text_lower) == stem_word(&query_lower) {
                            *score += 0.15;
                            bd.exact_title = 0.15;
                        } else if text_lower.starts_with(&query_lower) {
                            // Title starts with query: +0.08 (hybrid equivalent of +4.0 in keyword)
                            *score += 0.08;
                            bd.title_starts_with = 0.08;
                        } else if query_lower.starts_with(&text_lower) {
                            // Query starts with title: +0.08 (handles "design patterns" → "Design Pattern")
                            *score += 0.08;
                            bd.title_starts_with = 0.08;
                        } else if text_lower.contains(&query_lower) {
                            // Title contains query: +0.04 (hybrid equivalent of +2.0 in keyword)
                            *score += 0.04;
                            bd.title_contains = 0.04;
                        }
                    }
                    "tags" => {
                        let text_lower = field.text.to_lowercase();
                        // Proportional tag overlap (uses current score which includes title bonuses)
                        let matched = usize_to_f64(
                            query_tokens
                                .iter()
                                .filter(|qt| text_lower.contains(qt.as_str()))
                                .count(),
                        );
                        if matched > 0.0 {
                            let overlap = (matched / query_word_count) * 0.1 * *score;
                            *score += overlap;
                            bd.tag_overlap = overlap;
                        }
                    }
                    "id"
                        // Exact ID match: +0.10 (uses raw query, not stemmed tokens)
                        if field.text.to_lowercase() == query_lower => {
                            *score += 0.10;
                            bd.exact_id = 0.10;
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
    use super::super::field_model::Field;
    use super::*;

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

    // ── rerank_boost condition tests ──────────────────────────────

    #[test]
    fn test_rerank_boost_exact_title() {
        let doc = IndexedDoc {
            id: "test".to_string(),
            fields: vec![
                Field::new("title", "design pattern", 1.0),
                Field::new("body", "irrelevant", 0.5),
            ],
        };
        let boost = rerank_boost(
            &doc,
            "design pattern",
            &["design".to_string(), "pattern".to_string()],
        );
        assert_eq!(boost, 8.0, "exact title match should give +8.0");
    }

    #[test]
    fn test_rerank_boost_starts_with() {
        let doc = IndexedDoc {
            id: "test".to_string(),
            fields: vec![Field::new("title", "Design Patterns Reference", 1.0)],
        };
        let boost = rerank_boost(
            &doc,
            "design pattern",
            &["design".to_string(), "pattern".to_string()],
        );
        assert!(
            boost >= 4.0,
            "expected starts_with boost >= 4.0, got {}",
            boost
        );
    }

    #[test]
    fn test_rerank_boost_contains() {
        let doc = IndexedDoc {
            id: "test".to_string(),
            fields: vec![Field::new("title", "My Design Pattern Collection", 1.0)],
        };
        let boost = rerank_boost(&doc, "design pattern", &[]);
        assert_eq!(boost, 2.0, "title contains query should give +2.0");
    }

    #[test]
    fn test_rerank_boost_exact_id() {
        let doc = IndexedDoc {
            id: "wiki:reference:design-patterns".to_string(),
            fields: vec![
                Field::new("id", "wiki:reference:design-patterns", 0.0),
                Field::new("title", "irrelevant", 1.0),
            ],
        };
        let boost = rerank_boost(&doc, "wiki:reference:design-patterns", &[]);
        assert_eq!(boost, 7.0, "exact ID match should give +7.0");
    }

    #[test]
    fn test_rerank_boost_tag_overlap() {
        let doc = IndexedDoc {
            id: "test".to_string(),
            fields: vec![
                Field::new("title", "Some Page", 1.0),
                Field::new("tags", "design pattern reference", 0.0),
            ],
        };
        let boost = rerank_boost(
            &doc,
            "design pattern",
            &["design".to_string(), "pattern".to_string()],
        );
        assert_eq!(
            boost, 3.0,
            "tag overlap should give +3.0 (first matching token)"
        );
    }

    #[test]
    fn test_rerank_boost_combined_title_and_tags() {
        // Title starts_with (+4) + tag overlap (+3) = +7
        let doc = IndexedDoc {
            id: "test".to_string(),
            fields: vec![
                Field::new("title", "Design Patterns Reference", 1.0),
                Field::new("tags", "design pattern", 0.0),
            ],
        };
        let boost = rerank_boost(
            &doc,
            "design pattern",
            &["design".to_string(), "pattern".to_string()],
        );
        assert_eq!(boost, 7.0, "starts_with + tag overlap = 4 + 3");
    }

    #[test]
    fn test_rerank_boost_no_match() {
        let doc = IndexedDoc {
            id: "test".to_string(),
            fields: vec![
                Field::new("title", "Unrelated Page", 1.0),
                Field::new("tags", "foo bar", 0.0),
            ],
        };
        let boost = rerank_boost(
            &doc,
            "design pattern",
            &["design".to_string(), "pattern".to_string()],
        );
        assert_eq!(boost, 0.0, "no match should give 0");
    }

    #[test]
    fn test_rerank_boost_trailing_space() {
        // Trailing space breaks exact match because "design pattern " ≠ "design pattern"
        // and "design pattern".starts_with("design pattern ") is false (query is longer).
        // Trimming is done at the entry points (MCP/CLI), not inside rerank_boost.
        let doc = IndexedDoc {
            id: "test".to_string(),
            fields: vec![Field::new("title", "design pattern", 1.0)],
        };
        let boost_clean = rerank_boost(&doc, "design pattern", &[]);
        assert_eq!(boost_clean, 8.0, "exact match without space gives +8.0");

        let boost_trail = rerank_boost(&doc, "design pattern ", &[]);
        // "design pattern " starts with "design pattern" (reverse direction) → +4.0
        assert_eq!(
            boost_trail, 4.0,
            "trailing space triggers reverse starts_with (+4.0)"
        );
    }

    // ── Stemmed exact match tests ────────────────────────────────

    #[test]
    fn test_rerank_exact_match_via_stemming_plural() {
        // "design patterns" vs "Design Pattern" — Snowball stems both to "design pattern"
        let doc = IndexedDoc {
            id: "test".to_string(),
            fields: vec![Field::new("title", "Design Pattern", 1.0)],
        };
        let boost = rerank_boost(&doc, "design patterns", &[]);
        assert_eq!(
            boost, 8.0,
            "stemmed exact: patterns→pattern matches Pattern→pattern"
        );
    }

    #[test]
    fn test_rerank_exact_match_via_stemming_ing() {
        // "styling" vs "Style" — Snowball stems both to "style"
        // Note: stem_word() stems the full string as one word, so multi-word titles
        // work best with word-by-word tokenization, but single-word exact matches
        // exercise the same stem_word code path.
        let doc = IndexedDoc {
            id: "test".to_string(),
            fields: vec![Field::new("title", "Styling", 1.0)],
        };
        let boost = rerank_boost(&doc, "styling", &[]);
        assert_eq!(boost, 8.0, "raw exact: styling==Styling (case-insensitive)");

        let boost_stem = rerank_boost(&doc, "style", &[]);
        assert_eq!(
            boost_stem, 8.0,
            "stemmed exact: style stems to style == Styling stems to style"
        );
    }

    #[test]
    fn test_rerank_exact_match_via_stemming_er() {
        // "designer" vs "Design" — Snowball stems both to "design"
        let doc = IndexedDoc {
            id: "test".to_string(),
            fields: vec![Field::new("title", "Design", 1.0)],
        };
        let boost = rerank_boost(&doc, "designer", &[]);
        assert_eq!(
            boost, 8.0,
            "stemmed exact: designer→design matches Design→design"
        );
    }

    #[test]
    fn test_rerank_exact_match_via_stemming_ed() {
        // "rounded" vs "Round" — Snowball stems both to "round"
        let doc = IndexedDoc {
            id: "test".to_string(),
            fields: vec![Field::new("title", "Round", 1.0)],
        };
        let boost = rerank_boost(&doc, "rounded", &[]);
        assert_eq!(
            boost, 8.0,
            "stemmed exact: rounded→round matches Round→round"
        );
    }

    #[test]
    fn test_rerank_exact_match_raw_still_works() {
        // Raw exact match still takes priority when forms are identical
        let doc = IndexedDoc {
            id: "test".to_string(),
            fields: vec![Field::new("title", "design pattern", 1.0)],
        };
        let boost = rerank_boost(&doc, "design pattern", &[]);
        assert_eq!(boost, 8.0, "raw exact match still works");
    }

    // ── Stemming symmetry tests ──────────────────────────────────

    #[test]
    fn test_singular_query_matches_plural_doc() {
        // Doc has "patterns" (plural), query is "pattern" (singular)
        let mut index = Bm25Index::new();
        index.add_document(IndexedDoc {
            id: "patterns-doc".to_string(),
            fields: vec![
                Field::new("header", "Overview", 4.0),
                Field::new(
                    "body",
                    "This document discusses design patterns and their usage.",
                    1.0,
                ),
            ],
        });
        // Also add a doc without "pattern" at all — should not match
        index.add_document(IndexedDoc {
            id: "other-doc".to_string(),
            fields: vec![
                Field::new("header", "Overview", 4.0),
                Field::new("body", "Completely unrelated topic here.", 1.0),
            ],
        });

        let results = index.search("pattern", 10);
        assert!(
            !results.is_empty(),
            "should find result for 'pattern' query"
        );
        assert!(
            results.iter().any(|r| r.id == "patterns-doc"),
            "'pattern' should match doc containing 'patterns'"
        );
    }

    #[test]
    fn test_plural_query_matches_singular_doc() {
        // Doc has "pattern" (singular), query is "patterns" (plural)
        let mut index = Bm25Index::new();
        index.add_document(IndexedDoc {
            id: "pattern-doc".to_string(),
            fields: vec![
                Field::new("header", "Overview", 4.0),
                Field::new("body", "This is a single design pattern example.", 1.0),
            ],
        });
        index.add_document(IndexedDoc {
            id: "other-doc".to_string(),
            fields: vec![
                Field::new("header", "Overview", 4.0),
                Field::new("body", "Unrelated content.", 1.0),
            ],
        });

        let results = index.search("patterns", 10);
        assert!(
            !results.is_empty(),
            "should find result for 'patterns' query"
        );
        assert!(
            results.iter().any(|r| r.id == "pattern-doc"),
            "'patterns' should match doc containing 'pattern'"
        );
    }

    #[test]
    fn test_stemming_symmetry_scores() {
        // Both "pattern" and "patterns" queries should give the same score
        // for the same doc (within floating point tolerance)
        let mut index = Bm25Index::new();
        index.add_document(IndexedDoc {
            id: "doc1".to_string(),
            fields: vec![
                Field::new("header", "Overview", 4.0),
                Field::new("body", "Design patterns reference guide.", 1.0),
            ],
        });

        let results_singular = index.search("pattern", 10);
        let results_plural = index.search("patterns", 10);

        assert!(!results_singular.is_empty(), "should find for 'pattern'");
        assert!(!results_plural.is_empty(), "should find for 'patterns'");

        // Both should find the same doc with similar scores
        let score_singular = results_singular
            .iter()
            .find(|r| r.id == "doc1")
            .map(|r| r.score)
            .unwrap_or(0.0);
        let score_plural = results_plural
            .iter()
            .find(|r| r.id == "doc1")
            .map(|r| r.score)
            .unwrap_or(0.0);
        let ratio = if score_plural > 0.0 {
            score_singular / score_plural
        } else {
            0.0
        };
        assert!(
            (ratio - 1.0).abs() < 0.15,
            "scores should be similar: singular={} vs plural={} (ratio={})",
            score_singular,
            score_plural,
            ratio
        );
    }

    // ── Rerank + stemming integration ─────────────────────────────

    #[test]
    fn test_rerank_with_stemmed_title() {
        // Title has "Patterns" (plural), query is "pattern" (singular).
        // Rerank should still fire starts_with boost via raw query matching.
        let mut index = Bm25Index::new();
        index.add_document(IndexedDoc {
            id: "patterns".to_string(),
            fields: vec![
                Field::new("header", "Overview", 4.0),
                Field::new("body", "Content about design patterns.", 1.0),
                Field::new("title", "Design Patterns Reference", 0.0),
            ],
        });
        index.add_document(IndexedDoc {
            id: "other".to_string(),
            fields: vec![
                Field::new("header", "Overview", 4.0),
                Field::new("body", "Unrelated.", 1.0),
                Field::new("title", "Other Page", 0.0),
            ],
        });

        // Query with singular "pattern" — should still get rerank boost
        let results = index.search("design pattern", 10);
        assert!(!results.is_empty(), "should find results");
        let patterns_pos = results
            .iter()
            .position(|r| r.id == "patterns")
            .unwrap_or(usize::MAX);
        assert_eq!(
            patterns_pos, 0,
            "patterns doc should be #1 via rerank starts_with boost"
        );
    }

    // ── Edge cases ────────────────────────────────────────────────

    #[test]
    fn test_query_trailing_space_trimmed() {
        // Trailing spaces should not affect the search results
        let mut index = Bm25Index::new();
        index.add_document(IndexedDoc {
            id: "doc1".to_string(),
            fields: vec![
                Field::new("header", "Overview", 4.0),
                Field::new("body", "Design patterns explained.", 1.0),
            ],
        });

        let results_clean = index.search("design pattern", 10);
        let results_trailing = index.search("design pattern ", 10);

        assert_eq!(
            results_clean.len(),
            results_trailing.len(),
            "trailing space should not change result count"
        );
        // Scores should be similar (trailing space creates extra token but matching shouldn't change)
        for r_clean in &results_clean {
            if let Some(r_trail) = results_trailing.iter().find(|r| r.id == r_clean.id) {
                let diff = (r_clean.score - r_trail.score).abs();
                assert!(
                    diff < 0.01,
                    "trailing space changed score for {}: clean={} trail={}",
                    r_clean.id,
                    r_clean.score,
                    r_trail.score
                );
            }
        }
    }

    // ── Reported-bug regression tests ─────────────────────────────

    #[test]
    fn test_relevant_ranks_above_tangential() {
        // Simulate the reported bug: two pages, one matching both query terms
        // and one matching only one, with the title-based rerank boost.
        let mut index = Bm25Index::new();

        // Tangential page: body has "design" only, title doesn't start with query
        index.add_document(IndexedDoc {
            id: "tangential".to_string(),
            fields: vec![
                Field::new("header", "Overview", 4.0),
                Field::new("body", "Our design system uses a minimal aesthetic.", 1.0),
                Field::new("title", "Design Vocabulary", 0.0),
            ],
        });

        // Relevant page: body has "design patterns", title starts with query
        index.add_document(IndexedDoc {
            id: "relevant".to_string(),
            fields: vec![
                Field::new("header", "Overview", 4.0),
                Field::new(
                    "body",
                    "The classic GoF design patterns and DDD tactical patterns.",
                    1.0,
                ),
                Field::new("title", "Design Patterns Reference", 0.0),
            ],
        });

        let results = index.search("design pattern", 10);
        assert_eq!(results.len(), 2, "expected 2 results");

        let relevant_pos = results.iter().position(|r| r.id == "relevant").unwrap();
        let tangential_pos = results.iter().position(|r| r.id == "tangential").unwrap();
        assert!(
            relevant_pos < tangential_pos,
            "relevant page should rank above tangential page"
        );
    }

    #[test]
    fn test_bm25_add_then_search() {
        let mut index = Bm25Index::new();
        let doc = IndexedDoc {
            id: "searchable-doc".to_string(),
            fields: vec![
                Field::new("title", "Unique Term Doc", 1.0),
                Field::new(
                    "content",
                    "this document contains unique_search_term_xyz for testing",
                    0.5,
                ),
            ],
        };
        index.add_document(doc);

        let results = index.search("unique_search_term_xyz", 10);
        assert!(
            !results.is_empty(),
            "should find the document by unique term"
        );
    }
}
