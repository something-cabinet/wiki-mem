use std::sync::Arc;

use axum::extract::FromRef;
use axum::routing::{get, post};
use axum::Router;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

pub mod events;
pub mod graph;
pub mod health;
pub mod index;
pub mod initial;
pub mod lint;
pub mod memory;
pub mod pages;
pub mod search;
pub mod sources;
pub mod tasks;
pub mod templates;
pub mod time;
pub mod tools;
pub mod validate_mod;

#[derive(Clone)]
pub struct AppState {
    pub engine: Arc<wm_core::engine::EngineState>,
    pub registry: Arc<wm_core::ToolRegistry>,
}

impl FromRef<AppState> for Arc<wm_core::engine::EngineState> {
    fn from_ref(state: &AppState) -> Self {
        state.engine.clone()
    }
}

impl FromRef<AppState> for Arc<wm_core::ToolRegistry> {
    fn from_ref(state: &AppState) -> Self {
        state.registry.clone()
    }
}

pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/api/health", get(health::health))
        .route("/api/initial", post(initial::get))
        .route("/api/search/query", post(search::query))
        .route("/api/search/retrieve", post(search::retrieve))
        .route("/api/memory/list", post(memory::list))
        .route("/api/pages/list", post(pages::list))
        .route("/api/pages/get", post(pages::get))
        .route("/api/graph/stats", post(graph::stats))
        .route("/api/graph/full", post(graph::full))
        .route("/api/graph/neighbors", post(graph::neighbors))
        .route("/api/graph/path", post(graph::path))
        .route("/api/graph/subgraph", post(graph::subgraph))
        .route("/api/events", get(events::stream))
        .route("/api/pages/create", post(pages::create))
        .route("/api/pages/update", post(pages::update))
        .route("/api/pages/delete", post(pages::delete))
        .route("/api/tasks/board", post(tasks::board))
        .route("/api/tools/{name}", post(tools::call_tool))
        .route("/api/index/rebuild", post(index::rebuild))
        .route("/api/index/status", get(index::status))
        .route("/api/templates/list", get(templates::list))
        .route("/api/sources/list", get(sources::list))
        .route("/api/lint/check", post(lint::check))
        .route("/api/validate/check", post(validate_mod::check))
        .route("/api/time/report", get(time::report))
        .with_state(state)
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
}
