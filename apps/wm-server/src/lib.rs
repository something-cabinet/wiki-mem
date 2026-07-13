pub mod api;

use axum::{
    extract::State,
    http::Method,
    routing::{delete, get, post, put},
    Json, Router,
};
use std::sync::Arc;
use std::time::Duration;
use tower_http::cors::{Any, CorsLayer};
use tracing::info;
use wm_core::engine::EngineState;

#[derive(Clone)]
pub struct AppState {
    pub engine: Arc<EngineState>,
}

pub async fn handle_initial(
    State(state): State<AppState>,
) -> Json<serde_json::Value> {
    let snapshot = state.engine.graph.load();
    let graph = &snapshot.0;
    let elapsed = state.engine.started_at.elapsed().as_secs();

    Json(serde_json::json!({
        "success": true,
        "project": "WM Wiki Memory Engine",
        "graph_node_count": graph.node_count(),
        "graph_edge_count": graph.edge_count(),
        "session_memory_count": state.engine.session_memory.len(),
        "uptime_secs": elapsed,
        "stale": state.engine.stale_flag.load(std::sync::atomic::Ordering::Acquire),
    }))
}

pub async fn handle_health() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "ok",
        "service": "wm-server",
    }))
}

pub fn build_api_router(engine: Arc<EngineState>) -> Router {
    let state = AppState { engine };

    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_origin(Any)
        .allow_headers(Any);

    let api = Router::new()
        .route("/health", get(handle_health))
        .route("/initial", get(handle_initial))
        .route("/search", get(api::search::handle_search))
        .route("/pages", get(api::pages::list_pages))
        .route("/pages", post(api::pages::create_page))
        .route("/pages/{id}", get(api::pages::get_page))
        .route("/pages/{id}", put(api::pages::update_page))
        .route("/pages/{id}", delete(api::pages::delete_page))
        .route("/tasks/board", get(api::tasks::task_board))
        .route("/graph/stats", get(api::graph::graph_stats))
        .route("/graph/neighbors/{id}", get(api::graph::graph_neighbors))
        .route("/memory", get(api::memory::list_memory))
        .route("/events", get(api::events::event_stream));

    Router::new()
        .nest("/api", api)
        .layer(cors)
        .with_state(state)
}

/// Background task that rebuilds the graph when stale flag is set
async fn graph_rebuild_loop(engine: Arc<EngineState>) {
    let mut interval = tokio::time::interval(Duration::from_secs(10));
    loop {
        interval.tick().await;
        if engine.stale_flag.load(std::sync::atomic::Ordering::Acquire) {
            let root = engine
                .project_root
                .read()
                .map(|r| r.clone())
                .unwrap_or_else(|_| std::env::current_dir().unwrap_or_default());
            let wiki_dir = root.join(".wm").join("wiki");
            if wiki_dir.exists() {
                let count = engine.rebuild_graph(&wiki_dir);
                let sections = wm_core::graph::build_sections_from_wiki(&wiki_dir);
                let docs: Vec<wm_core::search::IndexedDoc> = sections
                    .iter()
                    .map(|s| wm_core::search::IndexedDoc {
                        id: s.section_id.clone(),
                        fields: vec![
                            wm_core::search::Field::new("header", &s.header, 4.0),
                            wm_core::search::Field::new("body", &s.body, 1.0),
                        ],
                    })
                    .collect();
                engine
                    .bm25_index
                    .store(Arc::new(wm_core::search::Bm25Index::build(docs)));
                engine.stale_flag.store(false, std::sync::atomic::Ordering::Release);
                info!("Background rebuild complete: {} pages", count);
            }
        }
    }
}

/// Start the HTTP API server
pub async fn run_server(engine: Arc<EngineState>, port: u16) -> Result<(), anyhow::Error> {
    // Spawn background graph rebuild task
    tokio::spawn(graph_rebuild_loop(engine.clone()));

    let app = build_api_router(engine);

    let addr = format!("127.0.0.1:{}", port);
    info!("Starting wm-server on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app.into_make_service()).await?;

    Ok(())
}
