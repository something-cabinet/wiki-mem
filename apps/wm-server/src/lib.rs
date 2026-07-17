pub mod api;

use axum::{
    extract::State,
    http::{Method, StatusCode, Uri},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::OnceLock;
use std::time::Duration;
use tower_http::cors::{Any, CorsLayer};
use tracing::info;
use wm_core::engine::EngineState;
use wm_core::mcp::tools;
use wm_core::mcp::transport::ToolRegistry;
use serde_json::Value;

#[derive(Clone)]
pub struct AppState {
    pub engine: Arc<EngineState>,
    pub registry: Arc<ToolRegistry>,
}

/// Global SPA static file state (initialized once by build_api_router_impl).
static SPA: OnceLock<SpaAssets> = OnceLock::new();

struct SpaAssets {
    dir: PathBuf,
    index_html: String,
}

/// Generic tool dispatch: POST /api/tools with JSON body `{"name":"wm_*", "arguments":{}}`.
pub async fn handle_tool_call(
    State(state): State<AppState>,
    Json(params): Json<Value>,
) -> Json<Value> {
    let name = params.get("name").and_then(|v| v.as_str()).unwrap_or("");
    let arguments = params.get("arguments").cloned().unwrap_or(Value::Object(Default::default()));
    match state.registry.dispatch(name, arguments) {
        Ok(result) => Json(result),
        Err(e) => Json(serde_json::json!({
            "success": false,
            "error": e.to_string(),
        })),
    }
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

/// SPA fallback handler: serve static files or index.html for client-side routing.
async fn handle_spa(uri: Uri) -> impl IntoResponse {
    let spa = match SPA.get() {
        Some(s) => s,
        None => return (StatusCode::NOT_FOUND, "Not found").into_response(),
    };
    let path = uri.path().trim_start_matches('/');
    let file_path = spa.dir.join(path);
    // Check if it's a real file (JS, CSS, assets, etc.)
    if file_path.starts_with(&spa.dir) && file_path.is_file() {
        let data = match std::fs::read(&file_path) {
            Ok(d) => d,
            Err(_) => return (StatusCode::NOT_FOUND, "Not found").into_response(),
        };
        let ext = path.rsplit('.').next().unwrap_or("");
        let mime = match ext {
            "js" => "application/javascript",
            "css" => "text/css",
            "html" => "text/html",
            "png" => "image/png",
            "svg" => "image/svg+xml",
            "ico" => "image/x-icon",
            "woff2" => "font/woff2",
            "json" => "application/json",
            _ => "application/octet-stream",
        };
        return (StatusCode::OK, [("content-type", mime)], data).into_response();
    }
    // SPA fallback: serve index.html with 200
    (StatusCode::OK, [("content-type", "text/html")], spa.index_html.as_bytes().to_vec()).into_response()
}

pub async fn handle_health() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "ok",
        "service": "wm-server",
    }))
}

/// Build an API router with a self-created ToolRegistry (convenience for standalone HTTP).
pub fn build_api_router(engine: Arc<EngineState>) -> Router {
    let mut reg = ToolRegistry::new();
    tools::register_all_tools(&mut reg, engine.clone());
    build_api_router_impl(engine, Arc::new(reg), None)
}

/// Build an API router with an externally provided ToolRegistry.
pub fn build_api_router_with(
    engine: Arc<EngineState>,
    registry: Arc<ToolRegistry>,
    web_dist: Option<PathBuf>,
) -> Router {
    build_api_router_impl(engine, registry, web_dist)
}

fn build_api_router_impl(
    engine: Arc<EngineState>,
    registry: Arc<ToolRegistry>,
    web_dist: Option<PathBuf>,
) -> Router {
    // Prepare SPA static files
    if let Some(dist) = web_dist {
        let browser_dir = dist.join("browser");
        let dir = if browser_dir.exists() { browser_dir } else { dist };
        if dir.exists() {
            let html = std::fs::read_to_string(dir.join("index.html")).unwrap_or_default();
            let _ = SPA.set(SpaAssets { dir, index_html: html });
            info!("Serving web UI from embedded directory");
        }
    }

    let state = AppState { engine, registry };

    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_origin(Any)
        .allow_headers(Any);

    let api = Router::new()
        .route("/tools", post(handle_tool_call))
        .route("/initial", post(handle_initial))
        .route("/search", post(api::search::handle_search))
        .route("/pages/list", post(api::pages::list_pages))
        .route("/pages/create", post(api::pages::create_page))
        .route("/pages/get", post(api::pages::get_page))
        .route("/pages/update", post(api::pages::update_page))
        .route("/pages/delete", post(api::pages::delete_page))
        .route("/tasks/board", post(api::tasks::task_board))
        .route("/graph/stats", post(api::graph::graph_stats))
        .route("/graph/neighbors", post(api::graph::graph_neighbors))
        .route("/memory/list", post(api::memory::list_memory))
        .route("/health", get(handle_health))
        .route("/events", get(api::events::event_stream));

    let api = api.layer(cors).with_state(state.clone());

    let mut router = Router::new().nest("/api", api);

    // SPA fallback route — serves static files or index.html for client-side routing.
    router = router.fallback(get(handle_spa));
    router
}

#[allow(unused_variables)]
fn try_serve_embedded_assets(router: &mut Router) {
    #[cfg(feature = "web-ui")]
    {
        use axum::response::{IntoResponse, Response};
        use axum::http::StatusCode;
        use rust_embed::RustEmbed;

        #[derive(RustEmbed)]
        #[folder = "../wm-web/dist/browser"]
        struct WebAssets;

        async fn embedded_fallback(path: axum::extract::Path<String>) -> Response {
            let request_path = path.0.trim_start_matches('/');
            let has_ext = request_path.contains('.');
            let asset = if has_ext {
                WebAssets::get(request_path)
            } else {
                None
            }
            .or_else(|| WebAssets::get("index.html"));

            match asset {
                Some(content) => {
                    let mime = mime_type(request_path);
                    Response::builder()
                        .header("content-type", mime)
                        .body(axum::body::Body::from(content.data))
                        .unwrap()
                }
                None => StatusCode::NOT_FOUND.into_response(),
            }
        }

        fn mime_type(path: &str) -> &'static str {
            if path.ends_with(".html") { "text/html" }
            else if path.ends_with(".js") { "application/javascript" }
            else if path.ends_with(".css") { "text/css" }
            else if path.ends_with(".png") { "image/png" }
            else if path.ends_with(".svg") { "image/svg+xml" }
            else if path.ends_with(".ico") { "image/x-icon" }
            else if path.ends_with(".woff2") { "font/woff2" }
            else if path.ends_with(".json") { "application/json" }
            else { "application/octet-stream" }
        }

        info!("Serving web UI from embedded assets");
        *router = std::mem::take::<Router>(router).fallback(embedded_fallback);
    }

    #[cfg(not(feature = "web-ui"))]
    {
        tracing::warn!(
            "Web UI not available. Run 'just build-web' or use 'ng serve' for development."
        );
    }
}

// ─── Background tasks ───────────────────────────────────

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

/// Start a background HTTP server on a random port with an externally provided ToolRegistry.
pub async fn start_background_server_with(
    engine: Arc<EngineState>,
    registry: Arc<ToolRegistry>,
    web_dist: Option<PathBuf>,
) -> Result<u16, anyhow::Error> {
    tokio::spawn(graph_rebuild_loop(engine.clone()));
    let app = build_api_router_with(engine, registry, web_dist);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
    let port = listener.local_addr()?.port();
    info!("Starting background wm-server on http://127.0.0.1:{}", port);
    tokio::spawn(async move {
        axum::serve(listener, app.into_make_service()).await.ok();
    });
    Ok(port)
}

/// Start a background HTTP server with a self-created ToolRegistry (convenience).
pub async fn start_background_server(engine: Arc<EngineState>) -> Result<u16, anyhow::Error> {
    let mut reg = ToolRegistry::new();
    tools::register_all_tools(&mut reg, engine.clone());
    start_background_server_with(engine, Arc::new(reg), None).await
}

pub async fn run_server(engine: Arc<EngineState>, port: u16) -> Result<(), anyhow::Error> {
    let mut reg = ToolRegistry::new();
    tools::register_all_tools(&mut reg, engine.clone());
    run_server_with(engine, Arc::new(reg), port, None).await
}

/// Run an HTTP server on a given port with an externally provided ToolRegistry.
pub async fn run_server_with(
    engine: Arc<EngineState>,
    registry: Arc<ToolRegistry>,
    port: u16,
    web_dist: Option<PathBuf>,
) -> Result<(), anyhow::Error> {
    tokio::spawn(graph_rebuild_loop(engine.clone()));
    let app = build_api_router_with(engine, registry, web_dist);
    let addr = format!("127.0.0.1:{}", port);
    info!("Starting wm-server on http://{}", addr);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app.into_make_service()).await?;
    Ok(())
}
