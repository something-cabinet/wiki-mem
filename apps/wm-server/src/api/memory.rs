use axum::{extract::State, Json};
use serde::Deserialize;
use serde_json::{json, Value};
use std::path::PathBuf;
use wm_core::engine::{EngineState, MemoryEntry};
use wm_core::status::MemoryStatus;
use crate::AppState;

#[derive(Deserialize)]
pub struct MemoryQuery {
    layer: Option<String>,
    status: Option<String>,
}

fn parse_status(s: &str) -> Option<MemoryStatus> {
    match s {
        "active" => Some(MemoryStatus::Active),
        "stale" => Some(MemoryStatus::Stale),
        "archived" => Some(MemoryStatus::Archived),
        _ => None,
    }
}

fn memory_dir(engine: &EngineState) -> PathBuf {
    engine
        .project_root
        .read()
        .map(|r| r.join(".wm").join("memory"))
        .unwrap_or_else(|_| PathBuf::from(".wm/memory"))
}

fn list_project_memory(engine: &EngineState, status_filter: Option<&MemoryStatus>) -> Vec<Value> {
    let dir = memory_dir(engine);
    if !dir.exists() {
        return Vec::new();
    }
    let mut result = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&dir) {
        for entry in entries.flatten() {
            if let Ok(content) = std::fs::read_to_string(entry.path()) {
                if let Ok(mem) = serde_json::from_str::<MemoryEntry>(&content) {
                    if status_filter.map_or(true, |sf| mem.status.as_ref() == Some(sf)) {
                        result.push(json!({
                            "id": mem.id,
                            "title": mem.title,
                            "content": mem.content,
                            "tags": mem.tags,
                            "created_at": mem.created_at,
                            "updated_at": mem.updated_at,
                            "status": mem.status,
                        }));
                    }
                }
            }
        }
    }
    result
}

pub async fn list_memory(
    State(state): State<AppState>,
    Json(params): Json<MemoryQuery>,
) -> Json<Value> {
    let engine = &state.engine;
    let layer = params.layer.as_deref().unwrap_or("project");
    let status_filter = params.status.as_deref().and_then(parse_status);
    match layer {
        "session" => {
            let entries: Vec<Value> = engine.session_memory.iter()
                .map(|entry| entry.value().clone())
                .filter(|e| status_filter.as_ref().map_or(true, |sf| e.status.as_ref() == Some(sf)))
                .map(|e| json!({
                    "id": e.id,
                    "title": e.title,
                    "content": e.content,
                    "tags": e.tags,
                    "created_at": e.created_at,
                    "updated_at": e.updated_at,
                    "status": e.status,
                }))
                .collect();
            Json(json!({ "success": true, "entries": entries, "total": entries.len() }))
        }
        "project" => {
            let entries = list_project_memory(engine, status_filter.as_ref());
            Json(json!({ "success": true, "entries": entries, "total": entries.len() }))
        }
        _ => Json(json!({ "success": false, "error": format!("Unknown layer: {layer}") })),
    }
}
