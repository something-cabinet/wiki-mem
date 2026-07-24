use std::sync::Arc;

use axum::extract::State;
use axum::Json;
use serde_json::{json, Value};

pub async fn list(
    State(state): State<Arc<wm_core::engine::EngineState>>,
    Json(_body): Json<Value>,
) -> Json<Value> {
    let page_type_filter = Some(wm_core::engine::PageType::Memory);
    match wm_core::page::list_pages(&state, page_type_filter.as_ref()) {
        Ok(pages) => Json(json!({"success": true, "entries": pages})),
        Err(e) => Json(json!({"success": false, "error": e.to_string()})),
    }
}
