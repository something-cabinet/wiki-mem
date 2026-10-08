use std::sync::atomic::Ordering;
use std::sync::Arc;

use wm_constants::{WIKI_DIR, WM_DIR};

use super::EngineState;

pub fn refresh_all(engine: &EngineState) {
    let root = engine
        .project_root
        .read()
        .map(|root| root.clone())
        .unwrap_or_default();
    let wiki_dir = root.join(WM_DIR).join(WIKI_DIR);
    if !wiki_dir.exists() {
        engine.stale_flag.store(false, Ordering::Release);
        return;
    }
    engine.rebuild_graph(&wiki_dir);
    let sections = Arc::new(crate::graph::build_sections_from_wiki(&wiki_dir));
    engine.section_corpus.store(sections.clone());
    let docs: Vec<crate::search::IndexedDoc> = sections
        .iter()
        .map(crate::search::indexed_doc_from_section)
        .collect();
    engine
        .bm25_index
        .store(Arc::new(crate::search::Bm25Index::build(docs)));
    engine.stale_flag.store(false, Ordering::Release);
}

pub fn refresh_all_if_stale(engine: &EngineState) -> bool {
    if !engine.stale_flag.load(Ordering::Acquire) {
        return false;
    }
    refresh_all(engine);
    true
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use wm_constants::{WIKI_DIR, WM_DIR};

    use super::*;

    fn engine_with_wiki() -> (tempfile::TempDir, Arc<EngineState>) {
        let dir = tempfile::tempdir().expect("tempdir");
        let root = dir.path().to_path_buf();
        let concepts = root.join(WM_DIR).join(WIKI_DIR).join("concepts");
        std::fs::create_dir_all(&concepts).expect("create wiki");
        std::fs::write(
            concepts.join("a.md"),
            "---\ntitle: A\ntype: concept\n---\n\n## Body\n\nalpha beta gamma\n",
        )
        .expect("write page");
        let config = crate::config::load_config(&root).unwrap_or_default();
        let (engine, _audit) = EngineState::new(config, root);
        (dir, Arc::new(engine))
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn refresh_clears_stale_only_after_full_rebuild() {
        let (_dir, engine) = engine_with_wiki();
        assert!(engine.stale_flag.load(Ordering::Acquire));
        assert!(refresh_all_if_stale(&engine));
        assert!(!engine.stale_flag.load(Ordering::Acquire));
        assert_eq!(engine.bm25_index.load().total_docs, 1);
        assert_eq!(engine.section_corpus.load().len(), 1);
        assert!(engine.graph.load().0.node_count() >= 1);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn refresh_is_a_noop_when_fresh() {
        let (_dir, engine) = engine_with_wiki();
        refresh_all(&engine);
        assert!(!refresh_all_if_stale(&engine));
    }
}
