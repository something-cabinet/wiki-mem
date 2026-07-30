use std::path::PathBuf;
use std::sync::OnceLock;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::response::{IntoResponse, Response};
use tower::ServiceExt;
use tower_http::services::ServeDir;

static SPA_DIR: OnceLock<PathBuf> = OnceLock::new();

/// Try to find the built Angular frontend directory.
/// Returns the directory that directly contains `index.html`.
pub fn find_dir(project_root: &std::path::Path) -> Option<PathBuf> {
    // Priority 1: monorepo dev/deployed path relative to project root
    for candidate in [
        // Angular 17+ application builder output
        project_root.join("apps").join("wm-web").join("dist").join("browser"),
        // Legacy Angular output
        project_root.join("apps").join("wm-web").join("dist"),
    ] {
        if candidate.join("index.html").exists() {
            return Some(candidate);
        }
    }

    // Priority 2: relative to the server binary (bundled in npm package or cargo install)
    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            for candidate in [
                parent.join("wm-web").join("dist").join("browser"),
                parent.join("wm-web").join("dist"),
            ] {
                if candidate.join("index.html").exists() {
                    return Some(candidate);
                }
            }
        }
    }

    None
}

/// Axum handler that serves the SPA with client-side routing fallback.
/// Non-file paths are rewritten to index.html for Angular client-side routing.
pub async fn handler(req: Request<Body>) -> Response {
    let spa_dir = match SPA_DIR.get() {
        Some(d) => d,
        None => return (StatusCode::NOT_FOUND, "Web UI not built").into_response(),
    };

    let path = req.uri().path().trim_start_matches('/');
    let file_path = spa_dir.join(if path.is_empty() { "index.html" } else { path });
    let serve_dir = ServeDir::new(spa_dir);

    if file_path.exists() && file_path.is_file() {
        match serve_dir.oneshot(req).await {
            Ok(resp) => return resp.into_response(),
            Err(e) => {
                return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response();
            }
        }
    }

    // SPA fallback: rewrite to index.html for client-side routing
    let index_req = match Request::builder()
        .uri("/index.html")
        .method(req.method().clone())
        .header("accept", "text/html")
        .body(req.into_body())
    {
        Ok(r) => r,
        Err(e) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response();
        }
    };

    match serve_dir.oneshot(index_req).await {
        Ok(resp) => resp.into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

/// Build a router that mounts the SPA as a fallback for all non-API routes.
///
/// The `spa_dir` should come from `find_dir()`. Pass `None` if the frontend
/// hasn't been built yet — the API-only router is returned as-is.
pub fn build_router(
    api_routes: axum::Router,
    spa_dir: Option<PathBuf>,
) -> axum::Router {
    if let Some(dir) = spa_dir {
        let _ = SPA_DIR.set(dir.clone());
        tracing::info!("Serving web UI from {}", dir.display());
    }

    api_routes.fallback(handler)
}
