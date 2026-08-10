//! Privileged MCP proxy channel (`/api/mcp/*`).
//!
//! This is the only write-capable HTTP surface of `wm-server`: it exposes the
//! full `ToolRegistry` (write tools included) to the MCP proxy. The boundary is
//! a separate credential — `.wm/state/mcp-token` — distinct from the read-only
//! web API token. There is deliberately NO allowlist / `WRITE_ACTIONS` guard on
//! this channel; possession of the MCP token is the authorization.
//!
//! Tool-level errors are returned as HTTP 200 with `{success:false, error,
//! code}` (the proxy maps them to `is_error`); only auth/transport failures
//! produce non-200 responses.

use std::sync::Arc;

use axum::extract::State;
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use axum::routing::post;
use axum::{Json, Router};
use serde_json::{json, Value};

use super::AppState;

const ERR_UNAUTHORIZED: &str = "Missing or invalid MCP token";

/// Sub-router for the MCP channel, guarded by its own mcp-token middleware.
///
/// The web token layer in `super::build_router` exempts `/api/mcp/*`, so this
/// middleware is the only credential check these routes see. The returned
/// `Router<AppState>` is merged into the outer router, which supplies the state
/// via its single `.with_state()` call.
pub fn router(state: AppState) -> Router<AppState> {
    let expected = state.mcp_token.clone();
    let project_root = state
        .engine
        .project_root
        .read()
        .map(|r| r.clone())
        .unwrap_or_default();
    Router::new()
        .route("/api/mcp/tools/list", post(tools_list))
        .route("/api/mcp/tools/call", post(tools_call))
        .layer(axum::middleware::from_fn(move |req, next| {
            require_mcp_token(expected.clone(), project_root.clone(), req, next)
        }))
}

async fn require_mcp_token(
    expected: Arc<String>,
    project_root: std::path::PathBuf,
    req: axum::extract::Request,
    next: Next,
) -> Response {
    let supplied = req
        .headers()
        .get(crate::web_token_service::header_name())
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default();

    if supplied != expected.as_str() {
        tracing::warn!("Rejected unauthenticated MCP request to {}", req.uri().path());
        let detail = format!("{} {}", req.method().as_str(), req.uri().path());
        wm_core::shared::audit_sink::audit_auth_failure(&project_root, &detail);
        return (StatusCode::UNAUTHORIZED, ERR_UNAUTHORIZED).into_response();
    }

    next.run(req).await
}

/// `POST /api/mcp/tools/list` — dynamic tool discovery.
///
/// Returns `ToolRegistry::list_tools()` verbatim (names, descriptions, input
/// schemas). There is no static tool list anywhere; the proxy fetches this.
pub async fn tools_list(State(registry): State<Arc<wm_core::ToolRegistry>>) -> Json<Value> {
    let tools = registry.list_tools();
    Json(json!({ "success": true, "data": { "tools": tools } }))
}

/// `POST /api/mcp/tools/call` — dispatch `{name, arguments}` to the registry.
///
/// Tool-level failures return HTTP 200 with `{success:false, error, code}`.
/// An unknown tool returns `{success:false, code:"METHOD_NOT_FOUND"}`. Only
/// auth failures (401) and transport failures are non-200.
pub async fn tools_call(
    State(registry): State<Arc<wm_core::ToolRegistry>>,
    Json(body): Json<Value>,
) -> Json<Value> {
    let name = match body.get("name").and_then(Value::as_str) {
        Some(name) if !name.is_empty() => name,
        _ => {
            return Json(json!({
                "success": false,
                "error": "missing or empty 'name' field",
                "code": "INVALID_PARAMS",
            }));
        }
    };

    if !registry.has_tool(name) {
        return Json(json!({
            "success": false,
            "error": format!("Unknown tool: {name}"),
            "code": "METHOD_NOT_FOUND",
        }));
    }

    let arguments = body
        .get("arguments")
        .cloned()
        .unwrap_or_else(|| json!({}));

    match registry.dispatch_async(name, arguments).await {
        Ok(data) => Json(json!({ "success": true, "data": data })),
        Err(e) => Json(json!({ "success": false, "error": e.message, "code": e.code })),
    }
}
