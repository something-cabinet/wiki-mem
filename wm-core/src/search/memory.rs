//! Memory index rebuild — walks `.wm/memory/*.json`, parses entries,
//! and builds a BM25 index with parallel file I/O.

use std::fs;

use rayon::prelude::*;

use super::index::{Bm25Index, Field, IndexedDoc};
use crate::engine::MemoryEntry;

/// Rebuild a BM25 index from memory JSON files in a directory.
/// Returns the built index and the number of entries indexed.
pub fn rebuild_memory_index_from_dir(memory_dir: &std::path::Path) -> (Bm25Index, usize) {
    if !memory_dir.exists() {
        return (Bm25Index::new(), 0);
    }

    // Collect paths (sequential walkdir — fast)
    let paths: Vec<_> = walkdir::WalkDir::new(memory_dir)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| {
            if let Err(err) = &e {
                tracing::warn!("Memory dir walk error: {}", err);
            }
            e.ok()
        })
        .filter(|e| e.file_type().is_file())
        .filter(|e| !e.path().extension().map(|ext| ext != "json").unwrap_or(true))
        .map(|e| e.path().to_path_buf())
        .collect();

    // Parallel: read + JSON parse each memory file
    let docs: Vec<IndexedDoc> = paths
        .par_iter()
        .filter_map(|path| {
            let content = fs::read_to_string(path).ok()?;
            let mem: MemoryEntry = serde_json::from_str(&content).ok()?;
            let doc_id = format!("memory:{}", mem.id);
            Some(IndexedDoc {
                id: doc_id,
                fields: vec![
                    Field::new("title", &mem.title, 4.0),
                    Field::new("body", &mem.content, 1.0),
                ],
            })
        })
        .collect();

    let index = Bm25Index::build(docs);
    let count = index.total_docs;
    (index, count)
}
