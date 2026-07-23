use axum::Json;
use serde_json::{json, Value};

/// `GET /api/health` – Returns server health status.
pub async fn health() -> Json<Value> {
    Json(json!({"success": true, "status": "ok"}))
}
