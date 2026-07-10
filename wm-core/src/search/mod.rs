//! Search engine — BM25, scoring, query orchestration, context retrieval.

pub mod index;
pub mod memory;
pub mod query;
pub mod retrieve;
pub mod scoring;

// Re-export public API at module level for backward compatibility
pub use index::{tokenize, Bm25Index, Field, IndexedDoc, SearchResult};
pub use memory::rebuild_memory_index_from_dir;
pub use query::{enrich_and_sort, merge_results_by_rrf, query, QueryParams, QueryResult};
pub use retrieve::retrieve_context;
pub use scoring::{cap_total_boost, recency_boost};

#[cfg(test)]
mod tests;
