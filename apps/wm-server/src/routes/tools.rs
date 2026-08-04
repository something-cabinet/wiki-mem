use axum::{
    extract::{Path, State},
    Json,
};
use serde_json::{json, Value};
use std::sync::Arc;

const READ_ONLY_TOOLS: &[&str] = &[
    "wm_initial",
    "wm_help",
    "wm_search",
    "wm_searchquery",
    "wm_searchretrieve",
    "wm_searchresolve",
    "wm_graph",
    "wm_graphstats",
    "wm_graphneighbors",
    "wm_graphpath",
    "wm_graphsubgraph",
    "wm_graphfull",
    "wm_code",
    "wm_codesearch",
    "wm_codesymbols",
    "wm_codedeps",
    "wm_codefile",
    "wm_lint",
    "wm_validate",
    "wm_log",
    "wm_reference",
    "wm_projectstatus",
    "wm_projectdetect",
];

const WRITE_ACTIONS: &[&str] = &[
    "create",
    "update",
    "delete",
    "remove",
    "add",
    "run",
    "rebuild",
    "embed",
    "link",
    "unlink",
    "start",
    "stop",
    "promote",
    "process",
    "complete",
    "error",
    "discover",
    "download",
    "rollback",
    "trigger",
    "check_ac",
    "uncheck_ac",
    "subtask",
    "clear",
    "fix",
];

const ERR_NOT_EXPOSED: &str = "Tool is not exposed over HTTP: the web API is read-only";

fn is_write_action(params: &Value) -> bool {
    params
        .get("action")
        .and_then(|a| a.as_str())
        .map(|a| WRITE_ACTIONS.contains(&a))
        .unwrap_or(false)
}

pub async fn call_tool(
    State(registry): State<Arc<wm_core::ToolRegistry>>,
    Path(tool_name): Path<String>,
    Json(params): Json<Value>,
) -> Json<Value> {
    if !READ_ONLY_TOOLS.contains(&tool_name.as_str()) {
        tracing::warn!("Blocked non-allowlisted tool over HTTP: {}", tool_name);
        return Json(json!({"success": false, "error": ERR_NOT_EXPOSED, "code": "FORBIDDEN"}));
    }

    if is_write_action(&params) {
        tracing::warn!("Blocked write action over HTTP: tool={}", tool_name);
        return Json(json!({"success": false, "error": ERR_NOT_EXPOSED, "code": "FORBIDDEN"}));
    }

    match registry.dispatch_async(&tool_name, params).await {
        Ok(result) => Json(json!({"success": true, "data": result})),
        Err(e) => Json(json!({"success": false, "error": e.message, "code": e.code})),
    }
}
