//! Search engine — BM25, scoring, query orchestration, context retrieval.

pub mod memory;
pub mod query;
pub mod retrieve;

pub use wm_search::{tokenize, Bm25Index, Field, IndexedDoc, SearchResult, cap_total_boost, recency_boost};
pub use query::{enrich_search_results_from_graph, merge_results_by_rrf, run_unified_search, QueryParams, QueryResult};
pub use retrieve::retrieve_context;

#[cfg(test)]
mod tests;
