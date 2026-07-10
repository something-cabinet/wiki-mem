use std::path::PathBuf;
use std::sync::Arc;

use schemars::JsonSchema;
use serde::Deserialize;
use crate::engine::{EngineState, MemoryEntry};
use crate::error::ToolError;
use crate::mcp::transport::ToolRegistry;
use crate::mcp::typed::TypedRegister;

// ─── Input types ────────────────────────────────────────────

#[derive(Deserialize, JsonSchema)]
struct WmMemoryListInput {
    #[schemars(description = "Filter by tag")]
    tag: Option<String>,
    #[schemars(description = "Filter by category")]
    category: Option<String>,
    #[schemars(description = "Max results")]
    limit: Option<usize>,
    #[schemars(description = "Memory layer: project/global/session")]
    layer: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
struct WmMemoryGetInput {
    #[schemars(description = "Memory entry ID")]
    id: String,
    #[schemars(description = "Memory layer: project/global/session")]
    layer: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
struct WmMemoryAddInput {
    #[schemars(description = "Memory entry ID")]
    id: String,
    #[schemars(description = "Title")]
    title: String,
    #[schemars(description = "Content")]
    content: String,
    #[schemars(description = "Tags")]
    tags: Option<Vec<String>>,
    #[schemars(description = "Memory layer: project/global/session")]
    layer: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
struct WmMemoryUpdateInput {
    #[schemars(description = "Memory entry ID")]
    id: String,
    #[schemars(description = "New title")]
    title: Option<String>,
    #[schemars(description = "New content")]
    content: Option<String>,
    #[schemars(description = "New tags")]
    tags: Option<Vec<String>>,
    #[schemars(description = "Memory layer: project/global/session")]
    layer: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
struct WmMemoryDeleteInput {
    #[schemars(description = "Memory entry ID to delete")]
    id: String,
    #[schemars(description = "Memory layer: project/global/session")]
    layer: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
struct WmMemoryPromoteInput {
    #[schemars(description = "Memory entry ID to promote")]
    id: String,
}

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
    registry.register_read(
        "wm_memory.list",
        "List memory entries from the selected memory layer",
        move |input: WmMemoryListInput| {
            let filter_tag = input.tag.map(|s| s.to_lowercase());
            let limit = input.limit.unwrap_or(50);
            let layer = input.layer.unwrap_or_else(|| "project".into());

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
        },
    );

    // ─── wm_memory.get ──────────────────────────────────────────────
    let e = engine.clone();
    registry.register_read(
        "wm_memory.get",
        "Get a single memory entry by ID from the selected memory layer",
        move |input: WmMemoryGetInput| {
            let id = input.id;
            let layer = input.layer.unwrap_or_else(|| "project".into());

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
        },
    );

    // ─── wm_memory.add ──────────────────────────────────────────────
    let e = engine.clone();
    registry.register_write(
        "wm_memory.add",
        "Create a new memory entry and write to the selected memory layer",
        move |input: WmMemoryAddInput| {
            let id = input.id;
            let title = input.title;
            let content = input.content;
            let tags = input.tags.unwrap_or_default();
            let layer = input.layer.unwrap_or_else(|| "project".into());

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
        },
    );

    // ─── wm_memory.update ───────────────────────────────────────────
    let e = engine.clone();
    registry.register_write(
        "wm_memory.update",
        "Update an existing memory entry in the selected memory layer. Only provided fields are changed.",
        move |input: WmMemoryUpdateInput| {
            let id = input.id;
            let layer = input.layer.unwrap_or_else(|| "project".into());

            let memory_dir = memory_dir(&layer, &e)?;
            let path = memory_dir.join(format!("{}.json", id));

            let content = std::fs::read_to_string(&path).map_err(|_| {
                ToolError::not_found("memory", &id)
            })?;

            let mut mem: MemoryEntry = serde_json::from_str(&content)
                .map_err(|e| ToolError::serde_error("deserialize memory", e))?;

            if let Some(title) = input.title {
                mem.title = title;
            }
            if let Some(content) = input.content {
                mem.content = content;
            }
            if let Some(tags) = input.tags {
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
        },
    );

    // ─── wm_memory.delete ───────────────────────────────────────────
    let e = engine.clone();
    registry.register_admin(
        "wm_memory.delete",
        "Delete a memory entry by ID from the selected memory layer",
        move |input: WmMemoryDeleteInput| {
            let id = input.id;
            let layer = input.layer.unwrap_or_else(|| "project".into());

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
        },
    );

    // ─── wm_memory.promote ───────────────────────────────────────────
    let e = engine.clone();
    registry.register_write(
        "wm_memory.promote",
        "Promote a memory entry from project layer to global layer (~/.wm/memory/)",
        move |input: WmMemoryPromoteInput| {
            let id = input.id;

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
        },
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
