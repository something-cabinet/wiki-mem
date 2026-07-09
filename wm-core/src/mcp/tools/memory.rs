use std::path::PathBuf;
use std::sync::Arc;

use serde_json::json;

use crate::engine::{EngineState, MemoryEntry};
use crate::error::ToolError;
use crate::mcp::handler::ToolArgs;
use crate::mcp::transport::ToolRegistry;

/// Resolve the memory directory for a given layer.
///
/// - `"project"` or `""` → `<project_root>/.wm/memory/`
/// - `"global"` → `~/.wm/memory/`
/// - `"session"` → error (ephemeral, not persisted)
fn memory_dir(layer: &str, engine: &EngineState) -> Result<PathBuf, ToolError> {
    match layer {
        "" | "project" => {
            let root = resolve_root(engine)?;
            Ok(root.join(".wm").join("memory"))
        }
        "global" => {
            let home = std::env::var("HOME")
                .or_else(|_| std::env::var("USERPROFILE"))
                .unwrap_or_else(|_| ".".into());
            Ok(PathBuf::from(home).join(".wm").join("memory"))
        }
        "session" => Err(ToolError::internal(
            "Session memory is ephemeral and not persisted",
        )),
        other => Err(ToolError::internal(format!(
            "Unknown memory layer: {}. Valid layers: project, global, session",
            other
        ))),
    }
}

/// Register memory tool handlers
pub fn register(registry: &mut ToolRegistry, engine: Arc<EngineState>) {
    // ─── wm_memory.list ─────────────────────────────────────────────
    let e = engine.clone();
    registry.register_with_schema(
        "memory.list",
        "List memory entries from the selected memory layer",
        json!({
            "type": "object",
            "properties": {
                "tag": { "type": "string", "description": "Filter by tag" },
                "category": { "type": "string", "description": "Filter by category" },
                "limit": { "type": "integer", "description": "Max results", "default": 50 },
                "layer": { "type": "string", "default": "project", "description": "Memory layer: project/global/session" }
            }
        }),
        Arc::new(move |params| {
            let args = ToolArgs::new(params);
            let filter_tag = args.optional_string("tag").map(|s| s.to_lowercase());
            let limit = args.optional_int("limit").unwrap_or(50);
            let layer = args.optional_string("layer").unwrap_or_else(|| "project".into());

            let memory_dir = memory_dir(&layer, &e)?;
            if !memory_dir.exists() || !memory_dir.is_dir() {
                return Ok(serde_json::json!({
                    "entries": [],
                    "total": 0,
                    "note": format!("{}/ not found", memory_dir.display())
                }));
            }

            let dir_entries = match std::fs::read_dir(&memory_dir) {
                Ok(entries) => entries,
                Err(e) => {
                    return Err(ToolError::io_error(
                        "read_dir",
                        memory_dir.to_string_lossy(),
                        e,
                    ))
                }
            };

            let mut entries: Vec<serde_json::Value> = Vec::new();
            let mut files: Vec<std::path::PathBuf> = Vec::new();

            for entry in dir_entries {
                let entry = match entry {
                    Ok(e) => e,
                    Err(_) => continue,
                };
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) != Some("json") {
                    continue;
                }
                files.push(path);
            }

            // Sort files for stable ordering
            files.sort();

            for path in &files {
                if entries.len() >= limit {
                    break;
                }

                let content = match std::fs::read_to_string(path) {
                    Ok(c) => c,
                    Err(_) => continue,
                };

                let mem: MemoryEntry = match serde_json::from_str(&content) {
                    Ok(m) => m,
                    Err(_) => continue,
                };

                // Filter by tag
                if let Some(ref tag) = filter_tag {
                    let has_tag = mem.tags.iter().any(|t| t.to_lowercase() == *tag);
                    if !has_tag {
                        continue;
                    }
                }

                entries.push(serde_json::json!({
                    "id": mem.id,
                    "title": mem.title,
                    "content": mem.content,
                    "tags": mem.tags,
                    "createdAt": mem.created_at,
                    "updatedAt": mem.updated_at,
                }));
            }

            Ok(serde_json::json!({
                "entries": entries,
                "total": entries.len(),
            }))
        }),
    );

    // ─── wm_memory.get ──────────────────────────────────────────────
    let e = engine.clone();
    registry.register_with_schema(
        "memory.get",
        "Get a single memory entry by ID from the selected memory layer",
        json!({
            "type": "object",
            "properties": {
                "id": { "type": "string", "description": "Memory entry ID" },
                "layer": { "type": "string", "default": "project", "description": "Memory layer: project/global/session" }
            },
            "required": ["id"]
        }),
        Arc::new(move |params| {
            let args = ToolArgs::new(params);
            let id = args.require_string("id")?;
            let layer = args.optional_string("layer").unwrap_or_else(|| "project".into());

            let memory_dir = memory_dir(&layer, &e)?;
            let path = memory_dir.join(format!("{}.json", id));

            let content = std::fs::read_to_string(&path).map_err(|_| {
                ToolError::not_found("memory", &id)
            })?;

            let mem: MemoryEntry = serde_json::from_str(&content)
                .map_err(|e| ToolError::serde_error("deserialize memory", e))?;

            Ok(serde_json::json!({
                "id": mem.id,
                "title": mem.title,
                "content": mem.content,
                "tags": mem.tags,
                "createdAt": mem.created_at,
                "updatedAt": mem.updated_at,
            }))
        }),
    );

    // ─── wm_memory.add ──────────────────────────────────────────────
    let e = engine.clone();
    registry.register_with_schema(
        "memory.add",
        "Create a new memory entry and write to the selected memory layer",
        json!({
            "type": "object",
            "properties": {
                "id": { "type": "string", "description": "Memory entry ID" },
                "title": { "type": "string", "description": "Title" },
                "content": { "type": "string", "description": "Content" },
                "tags": { "type": "array", "items": { "type": "string" }, "description": "Tags" },
                "layer": { "type": "string", "default": "project", "description": "Memory layer: project/global/session" }
            },
            "required": ["id", "title", "content"]
        }),
        Arc::new(move |params| {
            let args = ToolArgs::new(params);
            let id = args.require_string("id")?;
            let title = args.require_string("title")?;
            let content = args.require_string("content")?;
            let tags = args.optional_string_array("tags");
            let layer = args.optional_string("layer").unwrap_or_else(|| "project".into());

            let now = iso_now();
            let mem = MemoryEntry {
                id: id.clone(),
                title,
                content,
                tags,
                created_at: now.clone(),
                updated_at: now,
            };

            let memory_dir = memory_dir(&layer, &e)?;
            std::fs::create_dir_all(&memory_dir)
                .map_err(|e| ToolError::io_error("create_dir", memory_dir.to_string_lossy(), e))?;

            let path = memory_dir.join(format!("{}.json", id));
            let json = serde_json::to_string_pretty(&mem)
                .map_err(|e| ToolError::serde_error("serialize memory", e))?;
            std::fs::write(&path, &json)
                .map_err(|e| ToolError::io_error("write", path.to_string_lossy(), e))?;

            Ok(serde_json::json!({
                "id": mem.id,
                "title": mem.title,
                "content": mem.content,
                "tags": mem.tags,
                "createdAt": mem.created_at,
                "updatedAt": mem.updated_at,
            }))
        }),
    );

    // ─── wm_memory.update ───────────────────────────────────────────
    let e = engine.clone();
    registry.register_with_schema(
        "memory.update",
        "Update an existing memory entry in the selected memory layer. Only provided fields are changed.",
        json!({
            "type": "object",
            "properties": {
                "id": { "type": "string", "description": "Memory entry ID" },
                "title": { "type": "string", "description": "New title" },
                "content": { "type": "string", "description": "New content" },
                "tags": { "type": "array", "items": { "type": "string" }, "description": "New tags" },
                "layer": { "type": "string", "default": "project", "description": "Memory layer: project/global/session" }
            },
            "required": ["id"]
        }),
        Arc::new(move |params| {
            // Clone params so we can also inspect it directly after creating ToolArgs
            let args = ToolArgs::new(params.clone());
            let id = args.require_string("id")?;
            let layer = args.optional_string("layer").unwrap_or_else(|| "project".into());

            let memory_dir = memory_dir(&layer, &e)?;
            let path = memory_dir.join(format!("{}.json", id));

            let content = std::fs::read_to_string(&path).map_err(|_| {
                ToolError::not_found("memory", &id)
            })?;

            let mut mem: MemoryEntry = serde_json::from_str(&content)
                .map_err(|e| ToolError::serde_error("deserialize memory", e))?;

            if let Some(title) = args.optional_string("title") {
                mem.title = title;
            }
            if let Some(content) = args.optional_string("content") {
                mem.content = content;
            }
            // Check if "tags" key was explicitly provided (even if empty array)
            let tags = args.optional_string_array("tags");
            if params.get("tags").is_some() {
                mem.tags = tags;
            }
            mem.updated_at = iso_now();

            let json = serde_json::to_string_pretty(&mem)
                .map_err(|e| ToolError::serde_error("serialize memory", e))?;
            std::fs::write(&path, &json)
                .map_err(|e| ToolError::io_error("write", path.to_string_lossy(), e))?;

            Ok(serde_json::json!({
                "id": mem.id,
                "title": mem.title,
                "content": mem.content,
                "tags": mem.tags,
                "createdAt": mem.created_at,
                "updatedAt": mem.updated_at,
            }))
        }),
    );

    // ─── wm_memory.delete ───────────────────────────────────────────
    let e = engine.clone();
    registry.register_with_schema(
        "memory.delete",
        "Delete a memory entry by ID from the selected memory layer",
        json!({
            "type": "object",
            "properties": {
                "id": { "type": "string", "description": "Memory entry ID to delete" },
                "layer": { "type": "string", "default": "project", "description": "Memory layer: project/global/session" }
            },
            "required": ["id"]
        }),
        Arc::new(move |params| {
            let args = ToolArgs::new(params);
            let id = args.require_string("id")?;
            let layer = args.optional_string("layer").unwrap_or_else(|| "project".into());

            let memory_dir = memory_dir(&layer, &e)?;
            let path = memory_dir.join(format!("{}.json", id));

            if !path.exists() {
                return Err(ToolError::not_found("memory", &id));
            }

            std::fs::remove_file(&path)
                .map_err(|e| ToolError::io_error("delete", path.to_string_lossy(), e))?;

            Ok(serde_json::json!({
                "id": id,
                "status": "deleted"
            }))
        }),
    );

    // ─── wm_memory.promote ───────────────────────────────────────────
    let e = engine.clone();
    registry.register_with_schema(
        "memory.promote",
        "Promote a memory entry from project layer to global layer (~/.wm/memory/)",
        json!({
            "type": "object",
            "properties": {
                "id": { "type": "string", "description": "Memory entry ID to promote" }
            },
            "required": ["id"]
        }),
        Arc::new(move |params| {
            let args = ToolArgs::new(params);
            let id = args.require_string("id")?;

            let project_dir = memory_dir("project", &e)?;
            let project_path = project_dir.join(format!("{}.json", id));

            // Read from project layer
            let content = std::fs::read_to_string(&project_path).map_err(|_| {
                ToolError::not_found("memory", &id)
            })?;

            let mut mem: MemoryEntry = serde_json::from_str(&content)
                .map_err(|e| ToolError::serde_error("deserialize memory", e))?;

            // Update timestamp for the promotion
            mem.updated_at = iso_now();

            // Write to global layer
            let home = std::env::var("HOME")
                .or_else(|_| std::env::var("USERPROFILE"))
                .unwrap_or_else(|_| ".".into());
            let global_dir = PathBuf::from(home).join(".wm").join("memory");
            std::fs::create_dir_all(&global_dir)
                .map_err(|e| ToolError::io_error("create_dir", global_dir.to_string_lossy(), e))?;

            let global_path = global_dir.join(format!("{}.json", id));
            let json = serde_json::to_string_pretty(&mem)
                .map_err(|e| ToolError::serde_error("serialize memory", e))?;
            std::fs::write(&global_path, &json)
                .map_err(|e| ToolError::io_error("write", global_path.to_string_lossy(), e))?;

            Ok(serde_json::json!({
                "id": mem.id,
                "title": mem.title,
                "content": mem.content,
                "tags": mem.tags,
                "createdAt": mem.created_at,
                "updatedAt": mem.updated_at,
                "status": "promoted",
                "source": "project",
                "target": "global"
            }))
        }),
    );
}

/// Resolve the project root from engine state or fallback to current directory.
fn resolve_root(engine: &EngineState) -> Result<std::path::PathBuf, ToolError> {
    engine
        .project_root
        .read()
        .map(|r| r.clone())
        .or_else(|_| Ok(std::env::current_dir().map_err(|e| ToolError::internal(e.to_string()))?))
}

/// Return current datetime as ISO-8601 string (local time, second precision).
fn iso_now() -> String {
    chrono::Local::now().format("%Y-%m-%dT%H:%M:%S").to_string()
}
