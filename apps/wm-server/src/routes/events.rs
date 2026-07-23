use std::sync::Arc;

use axum::extract::State;
use axum::Json;
use serde_json::{json, Value};

/// `GET /api/events` – Stream real-time wiki events.
pub async fn stream(
    State(_state): State<Arc<wm_core::engine::EngineState>>,
) -> Json<Value> {
    Json(json!({"success": true, "events": []}))
}
