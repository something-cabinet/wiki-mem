mod commands;

use std::sync::Arc;
use wm_core::engine::{EngineState, MainEngine};
use wm_core::config::ProjectConfig;
use std::path::PathBuf;
use std::time::Duration;
use tauri::Manager;

fn detect_project_root() -> PathBuf {
    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let root = detect_project_root();
    let wiki_dir = root.join(".wm").join("wiki");
    let engine = Arc::new(MainEngine::new(ProjectConfig::default()));

    // Initial graph rebuild
    if wiki_dir.exists() {
        let ct = engine.state.config.read().unwrap().custom_edge_types.clone();
        wm_core::graph::rebuild_graph_snapshot(&engine.state.graph, &wiki_dir, &ct);
        engine.state.stale_flag.store(false, std::sync::atomic::Ordering::Release);
    }

    let engine_state = engine.state.clone();

    tauri::Builder::default()
        .manage(engine_state.clone())
        .invoke_handler(tauri::generate_handler![
            commands::get_initial,
            commands::search,
            commands::get_graph_full,
            commands::get_graph_stats,
            commands::get_graph_neighbors,
        ])
        .setup(move |_app| {
            tokio::spawn(async move {
                graph_rebuild_loop(engine_state, wiki_dir).await;
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

async fn graph_rebuild_loop(engine: Arc<EngineState>, wiki_dir: std::path::PathBuf) {
    let mut interval = tokio::time::interval(Duration::from_secs(10));
    loop {
        interval.tick().await;
        if engine.stale_flag.load(std::sync::atomic::Ordering::Acquire) {
            if wiki_dir.exists() {
                let ct = engine.config.read().unwrap().custom_edge_types.clone();
                wm_core::graph::rebuild_graph_snapshot(&engine.graph, &wiki_dir, &ct);
                engine.stale_flag.store(false, std::sync::atomic::Ordering::Release);
                tracing::info!("Background graph rebuild complete");
            }
        }
    }
}
