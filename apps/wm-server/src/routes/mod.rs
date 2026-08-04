use std::sync::Arc;

use axum::extract::FromRef;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::Router;
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
    pub spa_dir: Option<Arc<std::path::PathBuf>>,
    pub token: Arc<String>,
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
    let token = state.token.clone();
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
        .route("/api/tasks/board", post(tasks::board))
        .route("/api/tools/{name}", post(tools::call_tool))
        .route("/api/index/status", get(index::status))
        .route("/api/templates/list", get(templates::list))
        .route("/api/sources/list", get(sources::list))
        .route("/api/lint/check", post(lint::check))
        .route("/api/validate/check", post(validate_mod::check))
        .route("/api/time/report", get(time::report))
        .with_state(state)
        .layer(axum::middleware::from_fn(move |req, next| {
            require_token(token.clone(), req, next)
        }))
        .layer(axum::middleware::from_fn(reject_cross_site))
        .layer(TraceLayer::new_for_http())
}

const ERR_CROSS_SITE: &str = "Cross-site requests are not permitted";
const ERR_UNAUTHORIZED: &str = "Missing or invalid web API token";
const HEALTH_PATH: &str = "/api/health";

async fn require_token(
    expected: Arc<String>,
    req: axum::extract::Request,
    next: axum::middleware::Next,
) -> axum::response::Response {
    if req.uri().path() == HEALTH_PATH {
        return next.run(req).await;
    }

    let supplied = req
        .headers()
        .get(crate::web_token_service::header_name())
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default();

    if supplied != expected.as_str() {
        tracing::warn!("Rejected unauthenticated request to {}", req.uri().path());
        return (axum::http::StatusCode::UNAUTHORIZED, ERR_UNAUTHORIZED).into_response();
    }

    next.run(req).await
}

async fn reject_cross_site(
    req: axum::extract::Request,
    next: axum::middleware::Next,
) -> axum::response::Response {
    let headers = req.headers();
    let cross_site = headers
        .get("sec-fetch-site")
        .and_then(|v| v.to_str().ok())
        .map(|v| v == "cross-site" || v == "same-site")
        .unwrap_or(false);

    let foreign_origin = headers
        .get(axum::http::header::ORIGIN)
        .and_then(|v| v.to_str().ok())
        .map(|o| !o.contains(wm_constants::LOCALHOST_ADDR) && !o.contains("localhost"))
        .unwrap_or(false);

    if cross_site || foreign_origin {
        tracing::warn!("Rejected cross-site request to {}", req.uri().path());
        return (axum::http::StatusCode::FORBIDDEN, ERR_CROSS_SITE).into_response();
    }

    next.run(req).await
}
