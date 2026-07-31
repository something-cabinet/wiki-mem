use std::path::PathBuf;
use std::sync::OnceLock;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::response::{IntoResponse, Response};
use tower::ServiceExt;
use tower_http::services::ServeDir;

static SPA_DIR: OnceLock<PathBuf> = OnceLock::new();

pub fn find_dir(project_root: &std::path::Path) -> Option<PathBuf> {
    for candidate in [
        project_root
            .join("apps")
            .join("wm-web")
            .join("dist")
            .join("browser"),
        project_root.join("apps").join("wm-web").join("dist"),
    ] {
        if candidate.join("index.html").exists() {
            return Some(candidate);
        }
    }

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

pub fn build_router(api_routes: axum::Router, spa_dir: Option<PathBuf>) -> axum::Router {
    if let Some(dir) = spa_dir {
        let _ = SPA_DIR.set(dir.clone());
        tracing::info!("Serving web UI from {}", dir.display());
    }

    api_routes.fallback(handler)
}
