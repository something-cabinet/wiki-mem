use std::sync::Arc;

use crate::engine::{EngineState, MemoryEntry};
use crate::error::ToolError;
use crate::mcp::handler::ToolArgs;
use crate::mcp::transport::ToolRegistry;

/// Register memory tool handlers
pub fn register(registry: &mut ToolRegistry, engine: Arc<EngineState>) {
    // ─── wm_memory.list ─────────────────────────────────────────────
    let e = engine.clone();
    registry.register_with_desc(
        "wm_memory.list",
        "List memory entries from .wm/memory/*.json",
        Arc::new(move |params| {
            let args = ToolArgs::new(params);
            let filter_tag = args.optional_string("tag").map(|s| s.to_lowercase());
            let limit = args.optional_int("limit").unwrap_or(50);

            let root = resolve_root(&e)?;
            let memory_dir = root.join(".wm").join("memory");
            if !memory_dir.exists() || !memory_dir.is_dir() {
                return Ok(serde_json::json!({
                    "entries": [],
                    "total": 0,
                    "note": ".wm/memory/ not found"
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
                    "tags": mem.tags,
                    "created_at": mem.created_at,
                    "updated_at": mem.updated_at,
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
    registry.register_with_desc(
        "wm_memory.get",
        "Get a single memory entry by ID from .wm/memory/<id>.json",
        Arc::new(move |params| {
            let args = ToolArgs::new(params);
            let id = args.require_string("id")?;

            let root = resolve_root(&e)?;
            let path = root.join(".wm").join("memory").join(format!("{}.json", id));

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
                "created_at": mem.created_at,
                "updated_at": mem.updated_at,
            }))
        }),
    );

    // ─── wm_memory.add ──────────────────────────────────────────────
    let e = engine.clone();
    registry.register_with_desc(
        "wm_memory.add",
        "Create a new memory entry and write to .wm/memory/<id>.json",
        Arc::new(move |params| {
            let args = ToolArgs::new(params);
            let id = args.require_string("id")?;
            let title = args.require_string("title")?;
            let content = args.require_string("content")?;
            let tags = args.optional_string_array("tags");

            let now = iso_now();
            let mem = MemoryEntry {
                id: id.clone(),
                title,
                content,
                tags,
                created_at: now.clone(),
                updated_at: now,
            };

            let root = resolve_root(&e)?;
            let memory_dir = root.join(".wm").join("memory");
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
                "created_at": mem.created_at,
                "updated_at": mem.updated_at,
            }))
        }),
    );

    // ─── wm_memory.update ───────────────────────────────────────────
    let e = engine.clone();
    registry.register_with_desc(
        "wm_memory.update",
        "Update an existing memory entry. Only provided fields are changed.",
        Arc::new(move |params| {
            // Clone params so we can also inspect it directly after creating ToolArgs
            let args = ToolArgs::new(params.clone());
            let id = args.require_string("id")?;

            let root = resolve_root(&e)?;
            let path = root.join(".wm").join("memory").join(format!("{}.json", id));

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
                "created_at": mem.created_at,
                "updated_at": mem.updated_at,
            }))
        }),
    );

    // ─── wm_memory.delete ───────────────────────────────────────────
    let e = engine.clone();
    registry.register_with_desc(
        "wm_memory.delete",
        "Delete a memory entry by ID, removing .wm/memory/<id>.json",
        Arc::new(move |params| {
            let args = ToolArgs::new(params);
            let id = args.require_string("id")?;

            let root = resolve_root(&e)?;
            let path = root.join(".wm").join("memory").join(format!("{}.json", id));

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
