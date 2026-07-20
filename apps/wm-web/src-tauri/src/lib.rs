mod commands;

use std::sync::Arc;
use wm_core::engine::{EngineState, MainEngine};
use wm_core::config::ProjectConfig;
use std::path::PathBuf;
use std::time::Duration;

fn detect_project_root() -> PathBuf {
    // 1. Check WM_PROJECT env var (explicit override)
    if let Ok(path) = std::env::var("WM_PROJECT") {
        let p = PathBuf::from(path);
        if p.join(".wm").join("config.json").exists() {
            return p;
        }
    }

    // 2. Try the binary's own location (handles launching from target/debug/)
    if let Ok(exe_path) = std::env::current_exe() {
        let mut dir = exe_path.parent().map(|p| p.to_path_buf()).unwrap_or_default();
        let mut max_depth = 20;
        loop {
            if dir.join(".wm").join("config.json").exists() {
                return dir;
            }
            if max_depth == 0 || !dir.pop() {
                break;
            }
            max_depth -= 1;
        }
    }

    // 3. Fall back to walking up from current_dir
    let mut dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let mut max_depth = 20;
    loop {
        if dir.join(".wm").join("config.json").exists() {
            return dir;
        }
        if max_depth == 0 || !dir.pop() {
            return std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        }
        max_depth -= 1;
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let root = detect_project_root();
    let wiki_dir = root.join(".wm").join("wiki");
    let engine = Arc::new(MainEngine::with_root(ProjectConfig::default(), root.clone()));

    // Initial graph rebuild
    if wiki_dir.exists() {
        let ct = engine.state.config.read().unwrap().custom_edge_types.clone();
        wm_core::graph::rebuild_graph_snapshot(&engine.state.graph, &wiki_dir, &ct);
        engine.state.stale_flag.store(false, std::sync::atomic::Ordering::Release);
    }

    let engine_state = engine.state.clone();

    let builder = tauri::Builder::default()
        .manage(engine_state.clone())
        .invoke_handler(tauri::generate_handler![
            commands::get_initial,
            commands::search,
            commands::list_pages,
            commands::get_page,
            commands::create_page,
            commands::task_board,
            commands::list_memory,
            commands::get_graph_full,
            commands::get_graph_stats,
            commands::get_graph_neighbors,
            commands::compute_layout,
            commands::update_page,
            commands::delete_page,
            #[cfg(debug_assertions)]
            commands::get_captured_events,
            #[cfg(debug_assertions)]
            commands::clear_captured_events,
        ]);

    #[cfg(debug_assertions)]
    let builder = builder.plugin(tauri_plugin_pilot::init());

    builder
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
