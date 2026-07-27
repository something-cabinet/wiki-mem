use wm_search::Bm25Index;

pub fn rebuild_memory_index_from_dir(_memory_dir: &std::path::Path) -> (Bm25Index, usize) {
    (Bm25Index::new(), 0)
}
