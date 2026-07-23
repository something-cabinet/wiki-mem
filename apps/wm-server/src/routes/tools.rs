use axum::{
    extract::{Path, State},
    Json,
};
use serde_json::{json, Value};
use std::sync::Arc;

pub async fn call_tool(
    State(registry): State<Arc<wm_core::ToolRegistry>>,
    Path(tool_name): Path<String>,
    Json(params): Json<Value>,
) -> Json<Value> {
    match registry.dispatch_async(&tool_name, params).await {
        Ok(result) => Json(json!({"success": true, "data": result})),
        Err(e) => Json(json!({"success": false, "error": e.message, "code": e.code})),
    }
}
