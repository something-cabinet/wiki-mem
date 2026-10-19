use std::path::{Path, PathBuf};
use std::sync::Arc;

use wm_constants::{WIKI_DIR, WM_DIR};
use wm_core::config;
use wm_core::engine::{MainEngine, SectionDoc};

const HEADER_FIELD_WEIGHT: f64 = 4.0;
const BODY_FIELD_WEIGHT: f64 = 1.0;
const UNWEIGHTED_FIELD: f64 = 0.0;

pub fn create_engine() -> (Arc<MainEngine>, PathBuf) {
    let root = detect_root();
    let engine = build_engine(&root);
    (engine, root.join(WM_DIR).join(WIKI_DIR))
}

fn detect_root() -> PathBuf {
    config::detect_project_root().unwrap_or_else(|| PathBuf::from("."))
}

pub fn build_engine(root: &Path) -> Arc<MainEngine> {
    let engine = load_engine(root);
    migrate_legacy_memory(root, &engine);
    refresh_wiki_index(root, &engine);
    engine
}

fn load_engine(root: &Path) -> Arc<MainEngine> {
    let config = config::load_config(root).unwrap_or_default();
    Arc::new(MainEngine::with_root(config, root.to_path_buf()))
}

fn migrate_legacy_memory(root: &Path, engine: &Arc<MainEngine>) {
    if !root.join(WM_DIR).join("memory").exists() {
        return;
    }
    match wm_core::page::migrate_old_memory_json(&engine.state) {
        Ok(0) => {}
        Ok(count) => tracing::info!("Auto-migrated {} memory entries to wiki pages", count),
        Err(e) => tracing::warn!("Memory migration failed: {}", e),
    }
}

fn refresh_wiki_index(root: &Path, engine: &Arc<MainEngine>) {
    let wiki_dir = root.join(WM_DIR).join(WIKI_DIR);
    if !wiki_dir.exists() {
        return;
    }
    rebuild_from_engine(engine, &wiki_dir);
    engine
        .state
        .stale_flag
        .store(false, std::sync::atomic::Ordering::Release);
}

pub fn rebuild_from_engine(engine: &Arc<MainEngine>, wiki_dir: &Path) -> usize {
    let custom_edge_types = engine
        .state
        .config
        .read()
        .unwrap()
        .custom_edge_types
        .clone();
    let count = rebuild_graph(engine, wiki_dir, &custom_edge_types);
    rebuild_search_index(engine, wiki_dir);
    count
}

fn rebuild_graph(engine: &Arc<MainEngine>, wiki_dir: &Path, custom_edge_types: &[String]) -> usize {
    let count = engine.state.graph.rebuild(wiki_dir, custom_edge_types);
    let _ = wm_core::graph::auto_generate_index(wiki_dir, &engine.state.graph.snapshot().0);
    count
}

fn rebuild_search_index(engine: &Arc<MainEngine>, wiki_dir: &Path) {
    let sections = wm_core::graph::build_sections_from_wiki(wiki_dir);
    engine
        .state
        .index
        .replace_section_corpus(Arc::new(sections.clone()));
    let docs: Vec<wm_core::search::IndexedDoc> = sections
        .iter()
        .map(|s| wm_core::search::IndexedDoc {
            id: s.section_id.clone(),
            fields: indexed_fields(s),
        })
        .collect();
    let bm25 = wm_core::search::Bm25Index::build(docs);
    engine.state.index.replace_bm25_index(Arc::new(bm25));
}

fn indexed_fields(section: &SectionDoc) -> Vec<wm_core::search::Field> {
    vec![
        wm_core::search::Field::new("header", &section.header, HEADER_FIELD_WEIGHT),
        wm_core::search::Field::new("body", &section.body, BODY_FIELD_WEIGHT),
        wm_core::search::Field::new("id", &section.section_id, UNWEIGHTED_FIELD),
        wm_core::search::Field::new("title", &section.title, UNWEIGHTED_FIELD),
        wm_core::search::Field::new("tags", &section.tags.join(" "), UNWEIGHTED_FIELD),
    ]
}
