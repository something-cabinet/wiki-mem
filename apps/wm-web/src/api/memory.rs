use axum::{extract::{Query, State}, Json};
use serde::Deserialize;
use serde_json::{json, Value};
use std::path::PathBuf;
use wm_core::engine::{EngineState, MemoryEntry};
use crate::AppState;

#[derive(Deserialize)]
pub struct MemoryQuery {
    layer: Option<String>,
}

fn memory_dir(engine: &EngineState) -> PathBuf {
    engine
        .project_root
        .read()
        .map(|r| r.join(".wm").join("memory"))
        .unwrap_or_else(|_| PathBuf::from(".wm/memory"))
}

fn list_project_memory(engine: &EngineState) -> Vec<Value> {
    let dir = memory_dir(engine);
    if !dir.exists() {
        return Vec::new();
    }
    let mut result = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&dir) {
        for entry in entries.flatten() {
            if let Ok(content) = std::fs::read_to_string(entry.path()) {
                if let Ok(mem) = serde_json::from_str::<MemoryEntry>(&content) {
                    result.push(json!({
                        "id": mem.id,
                        "title": mem.title,
                        "content": mem.content,
                        "tags": mem.tags,
                        "created_at": mem.created_at,
                        "updated_at": mem.updated_at,
                    }));
                }
            }
        }
    }
    result
}

pub async fn list_memory(
    State(state): State<AppState>,
    Query(params): Query<MemoryQuery>,
) -> Json<Value> {
    let engine = &state.engine;
    let layer = params.layer.as_deref().unwrap_or("project");
    match layer {
        "session" => {
            let entries: Vec<Value> = engine.session_memory.iter().map(|entry| {
                let e = entry.value();
                json!({
                    "id": e.id,
                    "title": e.title,
                    "content": e.content,
                    "tags": e.tags,
                    "created_at": e.created_at,
                    "updated_at": e.updated_at,
                })
            }).collect();
            Json(json!({ "success": true, "entries": entries, "total": entries.len() }))
        }
        "project" => {
            let entries = list_project_memory(engine);
            Json(json!({ "success": true, "entries": entries, "total": entries.len() }))
        }
        _ => Json(json!({ "success": false, "error": format!("Unknown layer: {layer}") })),
    }
}
