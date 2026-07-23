use std::sync::Arc;
use axum::{extract::State, Json};
use serde_json::{json, Value};

pub async fn list(
    State(_state): State<Arc<wm_core::engine::EngineState>>,
) -> Json<Value> {
    Json(json!({"success": true, "templates": []}))
}
