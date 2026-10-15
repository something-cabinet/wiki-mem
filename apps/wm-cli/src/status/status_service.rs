use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::status::StatusArgs;
use crate::shared::services::rebuild_from_engine;
use crate::shared::traits::Command;
use wm_constants::{WIKI_DIR, WM_DIR};
use wm_core::config;
use wm_core::engine::{MainEngine, PageType};

const EMBEDDING_LOADED_LABEL: &str = "(loaded)";
const EMBEDDING_NOT_LOADED_LABEL: &str = "(not loaded)";

impl Command for StatusArgs {
    async fn run(self) -> Result<(), anyhow::Error> {
        run_status(self.json)
    }
}

fn run_status(json: bool) -> Result<(), anyhow::Error> {
    let (root, wiki_dir) = status_paths();
    let status = collect_status(&root, &wiki_dir);
    print_status(&status, &root, &wiki_dir, json)
}

fn status_paths() -> (PathBuf, PathBuf) {
    let root = config::detect_project_root().unwrap_or_else(|| PathBuf::from("."));
    let wiki_dir = root.join(WM_DIR).join(WIKI_DIR);
    (root, wiki_dir)
}

fn collect_status(root: &Path, wiki_dir: &Path) -> serde_json::Value {
    let mut status = serde_json::json!({
        "project_root": root,
        "wiki_dir": wiki_dir,
    });
    if wiki_dir.exists() {
        enrich_status(&mut status, wiki_dir);
    }
    status
}

fn enrich_status(status: &mut serde_json::Value, wiki_dir: &Path) {
    let engine = Arc::new(MainEngine::new());
    rebuild_from_engine(&engine, wiki_dir);
    let (nodes, edges, tasks) = graph_summary(&engine);
    status["graph_nodes"] = serde_json::json!(nodes);
    status["graph_edges"] = serde_json::json!(edges);
    status["sections"] = serde_json::json!(engine.state.section_corpus.load().len());
    status["bm25_docs"] = serde_json::json!(engine.state.bm25_index.load().total_docs);
    status["embedding_loaded"] = serde_json::json!(engine.state.embedder.is_loaded());
    status["embedding_model"] = serde_json::json!(engine.state.embedder.model_name().to_string());
    status["tasks_by_status"] = serde_json::json!(tasks);
}

fn graph_summary(engine: &MainEngine) -> (usize, usize, std::collections::BTreeMap<String, i32>) {
    let snapshot = engine.state.graph.load();
    let graph = &snapshot.0;
    let mut task_counts = std::collections::BTreeMap::new();
    for idx in graph.node_indices() {
        if graph[idx].page_type == PageType::Task {
            let state = format!("{:?}", graph[idx].status);
            *task_counts.entry(state).or_insert(0) += 1;
        }
    }
    (graph.node_count(), graph.edge_count(), task_counts)
}

fn print_status(
    status: &serde_json::Value,
    root: &Path,
    wiki_dir: &Path,
    json: bool,
) -> Result<(), anyhow::Error> {
    if json {
        println!("{}", serde_json::to_string_pretty(status)?);
        return Ok(());
    }
    print_status_text(status, root, wiki_dir);
    Ok(())
}

fn print_status_text(status: &serde_json::Value, root: &Path, wiki_dir: &Path) {
    println!("Wiki Memory Engine — Project Status");
    println!("  Project root:  {}", root.display());
    println!("  Wiki:          {}", wiki_dir.display());
    print_status_counter(status, "graph_nodes", "Graph nodes:  ");
    print_status_counter(status, "graph_edges", "Graph edges:  ");
    print_status_counter(status, "sections", "Sections:     ");
    print_status_counter(status, "bm25_docs", "BM25 docs:    ");
    print_status_embeddings(status);
    print_status_tasks(status);
}

fn print_status_counter(status: &serde_json::Value, key: &str, label: &str) {
    if let Some(value) = status.get(key).and_then(|v| v.as_u64()) {
        println!("  {} {}", label, value);
    }
}

fn print_status_embeddings(status: &serde_json::Value) {
    let Some(loaded) = status.get("embedding_loaded").and_then(|v| v.as_bool()) else {
        return;
    };
    let model = status
        .get("embedding_model")
        .and_then(|v| v.as_str())
        .unwrap_or("none");
    let label = embedding_label(loaded);
    println!("  Embeddings:    {} {}", model, label);
}

fn embedding_label(loaded: bool) -> &'static str {
    if loaded {
        return EMBEDDING_LOADED_LABEL;
    }
    EMBEDDING_NOT_LOADED_LABEL
}

fn print_status_tasks(status: &serde_json::Value) {
    let Some(tasks) = status.get("tasks_by_status").and_then(|v| v.as_object()) else {
        return;
    };
    println!("  Tasks:");
    for (state, count) in tasks {
        println!("    {}: {}", state, count);
    }
}
