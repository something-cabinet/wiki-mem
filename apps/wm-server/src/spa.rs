use std::path::PathBuf;
use std::sync::OnceLock;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::response::{IntoResponse, Response};
use tower::ServiceExt;
use tower_http::services::ServeDir;

static SPA_DIR: OnceLock<PathBuf> = OnceLock::new();
static SPA_TOKEN: OnceLock<String> = OnceLock::new();

const TOKEN_META_NAME: &str = "wm-token";
const HEAD_OPEN: &str = "<head>";

fn index_with_token(spa_dir: &std::path::Path) -> Option<String> {
    let html = std::fs::read_to_string(spa_dir.join("index.html")).ok()?;
    let token = SPA_TOKEN.get()?;
    let meta = format!(
        "<head>\n    <meta name=\"{}\" content=\"{}\">",
        TOKEN_META_NAME, token
    );
    Some(html.replacen(HEAD_OPEN, &meta, 1))
}

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

    let wants_index = path.is_empty() || path == "index.html";
    if wants_index {
        if let Some(html) = index_with_token(spa_dir) {
            return axum::response::Html(html).into_response();
        }
    }

    if file_path.exists() && file_path.is_file() {
        match serve_dir.oneshot(req).await {
            Ok(resp) => return resp.into_response(),
            Err(e) => {
                return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response();
            }
        }
    }

    let index_req = match Request::builder()        .uri("/index.html")
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

pub fn build_router(
    api_routes: axum::Router,
    spa_dir: Option<PathBuf>,
    token: String,
) -> axum::Router {
    let _ = SPA_TOKEN.set(token);
    if let Some(dir) = spa_dir {
        let _ = SPA_DIR.set(dir.clone());
        tracing::info!("Serving web UI from {}", dir.display());
    }

    api_routes.fallback(handler)
}
