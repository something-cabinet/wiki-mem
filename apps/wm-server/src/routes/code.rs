use std::sync::Arc;

use axum::extract::State;
use axum::Json;
use serde_json::{json, Value};

/// Dedicated read-only code-intel routes (replaces the generic `/api/tools/{name}`
/// dispatch endpoint). Each handler forwards to the matching `wm_code.*` tool and
/// wraps the result in the standard envelope.
const TOOL_SEARCH: &str = "wm_code.search";
const TOOL_SYMBOLS: &str = "wm_code.symbols";
const TOOL_FILE: &str = "wm_code.file";
const TOOL_DEPS: &str = "wm_code.deps";

async fn dispatch(registry: &wm_core::ToolRegistry, method: &str, params: Value) -> Json<Value> {
    match registry.dispatch_async(method, params).await {
        Ok(result) => Json(json!({"success": true, "data": result})),
        Err(e) => Json(json!({"success": false, "error": e.message, "code": e.code})),
    }
}

/// `POST /api/code/search` — `wm_code.search` (params: {path?, pattern?}).
pub async fn search(
    State(registry): State<Arc<wm_core::ToolRegistry>>,
    Json(params): Json<Value>,
) -> Json<Value> {
    dispatch(&registry, TOOL_SEARCH, params).await
}

/// `POST /api/code/symbols` — `wm_code.symbols` (params: {path?, name?}).
pub async fn symbols(
    State(registry): State<Arc<wm_core::ToolRegistry>>,
    Json(params): Json<Value>,
) -> Json<Value> {
    dispatch(&registry, TOOL_SYMBOLS, params).await
}

/// `POST /api/code/file` — `wm_code.file` (params: {path}).
pub async fn file(
    State(registry): State<Arc<wm_core::ToolRegistry>>,
    Json(params): Json<Value>,
) -> Json<Value> {
    dispatch(&registry, TOOL_FILE, params).await
}

/// `POST /api/code/deps` — `wm_code.deps` (params: {path?, depth?}).
pub async fn deps(
    State(registry): State<Arc<wm_core::ToolRegistry>>,
    Json(params): Json<Value>,
) -> Json<Value> {
    dispatch(&registry, TOOL_DEPS, params).await
}
