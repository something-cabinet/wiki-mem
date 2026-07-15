use axum::{
    extract::State,
    Json,
};
use serde::Deserialize;
use serde_json::{json, Value};
use std::sync::Arc;
use crate::AppState;


#[derive(Deserialize)]
pub struct CreatePayload {
    path: String,
    title: String,
    content: Option<String>,
    r#type: Option<String>,
    tags: Option<Vec<String>>,
    status: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdatePayload {
    id: Option<String>,
    title: Option<String>,
    content: Option<String>,
    r#type: Option<String>,
    status: Option<String>,
    tags: Option<Vec<String>>,
    append_notes: Option<String>,
}

pub async fn list_pages(
    State(state): State<AppState>,
) -> Json<Value> {
    let engine = &state.engine;
    match wm_core::page::list_pages(engine, None) {
        Ok(pages) => {
            let total = pages.len();
            Json(json!({ "success": true, "pages": pages, "total": total }))
        }
        Err(e) => Json(json!({ "success": false, "error": e.to_string() })),
    }
}

#[derive(Deserialize)]
pub struct GetPayload {
    id: String,
}

pub async fn get_page(
    State(state): State<AppState>,
    Json(payload): Json<GetPayload>,
) -> Json<Value> {
    let engine = &state.engine;
    match wm_core::page::get_page(engine, &payload.id) {
        Ok(content) => {
            let snapshot = engine.graph.load();
            let meta = snapshot.1.get(&payload.id).map(|&idx| &snapshot.0[idx]);
            Json(json!({
                "success": true,
                "page": meta,
                "content": content.raw,
                "sections": content.sections,
            }))
        }
        Err(e) => Json(json!({ "success": false, "error": e.to_string() })),
    }
}

pub async fn create_page(
    State(state): State<AppState>,
    Json(payload): Json<CreatePayload>,
) -> Json<Value> {
    let engine = &state.engine;
    let page_type = payload.r#type.unwrap_or_else(|| {
        let first = payload.path.split('/').next().unwrap_or("concept");
        match first {
            "tasks" => "task",
            "specs" => "spec",
            "concepts" => "concept",
            "patterns" => "pattern",
            "decisions" => "decision",
            "howto" => "howto",
            "reference" => "reference",
            _ => "concept",
        }
        .to_string()
    });

    let mut frontmatter = format!("title: {}\ntype: {}\n", payload.title, page_type);
    if let Some(status) = payload.status {
        frontmatter.push_str(&format!("status: {}\n", status));
    }
    if let Some(tags) = &payload.tags {
        if !tags.is_empty() {
            frontmatter.push_str(&format!("tags: [{}]\n", tags.join(", ")));
        }
    }

    let content = payload.content.unwrap_or_default();
    match wm_core::page::create_page(engine, &payload.path, &frontmatter, &content) {
        Ok(id) => {
            engine.index_scheduler.submit("page", {
                let e = engine.clone();
                move || {
                    let root = e
                        .project_root
                        .read()
                        .map(|r| r.clone())
                        .unwrap_or_else(|_| std::env::current_dir().unwrap_or_default());
                    let wiki_dir = root.join(".wm").join("wiki");
                    let sections = wm_core::graph::build_sections_from_wiki(&wiki_dir);
                    let docs: Vec<wm_core::search::IndexedDoc> = sections
                        .iter()
                        .map(|s| wm_core::search::IndexedDoc {
                            id: s.section_id.clone(),
                            fields: vec![
                                wm_core::search::Field::new("header", &s.header, 4.0),
                                wm_core::search::Field::new("body", &s.body, 1.0),
                            ],
                        })
                        .collect();
                    e.bm25_index
                        .store(Arc::new(wm_core::search::Bm25Index::build(docs)));
                }
            });
            Json(json!({ "success": true, "id": id, "path": payload.path }))
        }
        Err(e) => Json(json!({ "success": false, "error": e.to_string() })),
    }
}

pub async fn update_page(
    State(state): State<AppState>,
    Json(payload): Json<UpdatePayload>,
) -> Json<Value> {
    let engine = &state.engine;
    let page_id = payload.id.clone().unwrap_or_default();
    let params = wm_core::page::PageUpdateParams {
        title: payload.title,
        content: payload.content,
        status: payload.status,
        r#type: payload.r#type,
        tags: payload.tags.map(|t| t),
        append_notes: payload.append_notes,
        ..Default::default()
    };
    match wm_core::page::update_page(engine, &page_id, &params) {
        Ok(_) => Json(json!({ "success": true })),
        Err(e) => Json(json!({ "success": false, "error": e.to_string() })),
    }
}

#[derive(Deserialize)]
pub struct DeletePayload {
    id: String,
}

pub async fn delete_page(
    State(state): State<AppState>,
    Json(payload): Json<DeletePayload>,
) -> Json<Value> {
    let engine = &state.engine;
    match wm_core::page::delete_page(engine, &payload.id) {
        Ok(_) => Json(json!({ "success": true })),
        Err(e) => Json(json!({ "success": false, "error": e.to_string() })),
    }
}
