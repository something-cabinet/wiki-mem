use axum::{extract::State, Json};
use serde_json::{json, Value};
use std::sync::Arc;

pub async fn report(State(_state): State<Arc<wm_core::engine::EngineState>>) -> Json<Value> {
    Json(json!({"success": true, "entries": [], "total_seconds": 0}))
}
