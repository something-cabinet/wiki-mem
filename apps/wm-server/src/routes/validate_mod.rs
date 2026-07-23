use std::sync::Arc;
use axum::{extract::State, Json};
use serde_json::{json, Value};

pub async fn check(
    State(_state): State<Arc<wm_core::engine::EngineState>>,
    Json(_body): Json<Value>,
) -> Json<Value> {
    Json(json!({"success": true, "errors": [], "warnings": []}))
}
