use std::sync::Arc;

use axum::extract::State;
use axum::Json;
use serde::Deserialize;
use serde_json::{json, Value};
use wm_core::engine::{EngineState, PageType};

#[derive(Deserialize)]
pub struct ListInput {
    pub r#type: Option<String>,
}

pub async fn list(
    State(state): State<Arc<EngineState>>,
    Json(input): Json<ListInput>,
) -> Json<Value> {
    let page_type = input.r#type.as_deref().and_then(|t| {
        if t == "all" || t.is_empty() {
            None
        } else {
            serde_json::from_value::<PageType>(json!(t)).ok()
        }
    });

    match wm_core::page::list_pages(&state, page_type.as_ref()) {
        Ok(pages) => Json(json!({"success": true, "pages": pages})),
        Err(e) => Json(json!({"success": false, "error": e.to_string()})),
    }
}

#[derive(Deserialize)]
pub struct GetInput {
    pub id: String,
}

pub async fn get(
    State(state): State<Arc<EngineState>>,
    Json(input): Json<GetInput>,
) -> Json<Value> {
    match wm_core::page::get_page(&state, &input.id) {
        Ok(page) => {
            let meta = page.meta.as_ref().map(|m| {
                json!({
                    "id": m.id,
                    "title": m.title,
                    "type": m.page_type,
                    "status": m.status,
                    "tags": m.tags,
                    "created_at": m.created_at,
                    "updated_at": m.updated_at,
                })
            });
            Json(json!({
                "success": true,
                "page": {
                    "id": input.id,
                    "content": page.raw,
                    "meta": meta,
                }
            }))
        }
        Err(e) => Json(json!({"success": false, "error": e.to_string()})),
    }
}
