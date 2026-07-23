use std::sync::Arc;

use axum::extract::State;
use axum::Json;
use serde::Deserialize;
use serde_json::{json, Value};
use wm_core::engine::{EngineState, PageType};
use wm_core::page::PageUpdateParams;

#[derive(Deserialize)]
pub struct ListInput {
    pub r#type: Option<String>,
}

/// `POST /api/pages/list` – List wiki pages.
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

/// `POST /api/pages/get` – Get a single wiki page.
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

#[derive(Deserialize)]
pub struct CreateInput {
    pub path: String,
    pub title: String,
    pub content: Option<String>,
    pub r#type: Option<String>,
    #[allow(dead_code)]
    pub tags: Option<Vec<String>>,
}

/// `POST /api/pages/create` – Create a wiki page.
pub async fn create(
    State(state): State<Arc<EngineState>>,
    Json(input): Json<CreateInput>,
) -> Json<Value> {
    let content = input.content.unwrap_or_default();
    let page_type = input.r#type.as_deref().unwrap_or("note");
    let frontmatter = format!("---\ntitle: {}\ntype: {}\n---", input.title, page_type);

    match wm_core::page::create_page(&state, &input.path, &frontmatter, &content) {
        Ok(id) => Json(json!({"success": true, "id": id})),
        Err(e) => Json(json!({"success": false, "error": e.to_string()})),
    }
}

#[derive(Deserialize)]
pub struct UpdateInput {
    pub id: String,
    pub title: Option<String>,
    pub content: Option<String>,
    pub status: Option<String>,
    pub tags: Option<Vec<String>>,
}

/// `POST /api/pages/update` – Update a wiki page.
pub async fn update(
    State(state): State<Arc<EngineState>>,
    Json(input): Json<UpdateInput>,
) -> Json<Value> {
    let params = PageUpdateParams {
        title: input.title,
        content: input.content,
        status: input.status,
        tags: input.tags,
        ..Default::default()
    };
    match wm_core::page::update_page(&state, &input.id, &params) {
        Ok(_) => Json(json!({"success": true})),
        Err(e) => Json(json!({"success": false, "error": e.to_string()})),
    }
}

#[derive(Deserialize)]
pub struct DeleteInput {
    pub id: String,
}

/// `POST /api/pages/delete` – Delete a wiki page.
pub async fn delete(
    State(state): State<Arc<EngineState>>,
    Json(input): Json<DeleteInput>,
) -> Json<Value> {
    match wm_core::page::delete_page(&state, &input.id) {
        Ok(_) => Json(json!({"success": true})),
        Err(e) => Json(json!({"success": false, "error": e.to_string()})),
    }
}
