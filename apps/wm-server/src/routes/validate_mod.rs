use axum::{extract::State, Json};
use serde_json::{json, Value};
use std::sync::Arc;

pub async fn check(
    State(_state): State<Arc<wm_core::engine::EngineState>>,
    Json(_body): Json<Value>,
) -> Json<Value> {
    Json(json!({"success": true, "errors": [], "warnings": []}))
}
