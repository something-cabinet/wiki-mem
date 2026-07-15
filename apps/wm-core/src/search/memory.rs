//! Memory index rebuild — DEPRECATED.
//!
//! Memory entries are now stored as regular wiki pages with `page_type: memory`
//! in `.wm/wiki/memory/*.md`. The main BM25 index and graph handle indexing.
//! This module is kept as a stub for backward compatibility.

use super::index::Bm25Index;

/// Stub — memory entries are now wiki pages, indexed via the main pipeline.
/// Returns an empty index with count 0.
pub fn rebuild_memory_index_from_dir(_memory_dir: &std::path::Path) -> (Bm25Index, usize) {
    (Bm25Index::new(), 0)
}
