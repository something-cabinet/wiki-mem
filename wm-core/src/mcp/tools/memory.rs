use std::path::PathBuf;
use std::sync::Arc;

use std::collections::HashMap;
use dashmap::DashMap;
use schemars::JsonSchema;
use serde::Deserialize;
use crate::embed::EmbedVector;
use crate::engine::{EngineState, MemoryEntry};
use crate::error::ToolError;
use crate::mcp::transport::ToolRegistry;
use crate::mcp::typed::TypedRegister;
use crate::search::scoring::recency_boost;

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

/// Check if a layer string refers to session memory.
fn is_session(layer: &str) -> bool {
    layer == "session"
}

/// Resolve the memory directory for a given layer.
///
/// - `"project"` or `""` → `<project_root>/.wm/memory/`
/// - `"global"` → `~/.wm/memory/`
/// - `"session"` → returns Ok with an empty path (must use `is_session` to detect)
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
        "session" => Ok(PathBuf::new()), // sentinel — check is_session() first
        other => Err(ToolError::internal(format!(
            "Unknown memory layer: {}. Valid layers: project, global, session",
            other
        ))),
    }
}

/// Collect session memory entries, optionally filtered by tag.
fn session_entries(
    store: &DashMap<String, MemoryEntry>,
    filter_tag: Option<&str>,
    limit: usize,
) -> Vec<serde_json::Value> {
    let mut entries: Vec<serde_json::Value> = Vec::new();
    for item in store.iter() {
        if entries.len() >= limit {
            break;
        }
        let mem = item.value();
        if let Some(tag) = filter_tag {
            let has_tag = mem.tags.iter().any(|t| t.to_lowercase() == tag);
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
    entries
}

/// Maximum capacity of session memory before FSRS-based eviction.
const SESSION_CAPACITY: usize = 1000;

/// Evict the entry with the lowest FSRS recency score.
/// Uses `recency_boost` with fsrs model on the entry's age in days.
fn evict_lowest_fsrs(store: &DashMap<String, MemoryEntry>) {
    let now = chrono::Utc::now();
    let mut lowest_score = f64::MAX;
    let mut lowest_key = String::new();

    for item in store.iter() {
        let mem = item.value();
        let updated = chrono::DateTime::parse_from_rfc3339(&mem.updated_at)
            .map(|dt| dt.with_timezone(&chrono::Utc))
            .unwrap_or(now);
        let days = (now - updated).num_seconds() as f64 / 86400.0;
        let score = recency_boost(days, "fsrs", 7.0);
        if score < lowest_score {
            lowest_score = score;
            lowest_key = mem.id.clone();
        }
    }

    if !lowest_key.is_empty() {
        store.remove(&lowest_key);
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

            if is_session(&layer) {
                let entries = session_entries(&e.session_memory, filter_tag.as_deref(), limit);
                return Ok(serde_json::json!({
                    "entries": entries,
                    "total": entries.len(),
                }));
            }

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

            if is_session(&layer) {
                let mem = e.session_memory.get(&id).ok_or_else(|| {
                    ToolError::not_found("memory", &id)
                })?;
                return Ok(serde_json::json!({
                    "id": mem.id,
                    "title": mem.title,
                    "content": mem.content,
                    "tags": mem.tags,
                    "createdAt": mem.created_at,
                    "updatedAt": mem.updated_at,
                }));
            }

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

            if is_session(&layer) {
                if e.session_memory.len() >= SESSION_CAPACITY {
                    evict_lowest_fsrs(&e.session_memory);
                }
                embed_memory_entry(&e, &id, &mem.title, &mem.content);
                e.session_memory.insert(id.clone(), mem);
                return Ok(serde_json::json!({
                    "id": id,
                    "status": "created",
                    "layer": "session",
                }));
            }

            let memory_dir = memory_dir(&layer, &e)?;
            std::fs::create_dir_all(&memory_dir)
                .map_err(|e| ToolError::io_error("create_dir", memory_dir.to_string_lossy(), e))?;

            let mem_path = memory_dir.join(format!("{}.json", id));
            let json = serde_json::to_string_pretty(&mem)
                .map_err(|e| ToolError::serde_error("serialize memory", e))?;
            std::fs::write(&mem_path, &json)
                .map_err(|e| ToolError::io_error("write", mem_path.to_string_lossy(), e))?;

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

            if is_session(&layer) {
                let mut mem = e.session_memory.get_mut(&id).ok_or_else(|| {
                    ToolError::not_found("memory", &id)
                })?;
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
                embed_memory_entry(&e, &mem.id, &mem.title, &mem.content);
                return Ok(serde_json::json!({
                    "id": mem.id,
                    "title": mem.title,
                    "content": mem.content,
                    "tags": mem.tags,
                    "createdAt": mem.created_at,
                    "updatedAt": mem.updated_at,
                }));
            }

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

            if is_session(&layer) {
                remove_memory_vector(&e, &id);
                e.session_memory.remove(&id).ok_or_else(|| {
                    ToolError::not_found("memory", &id)
                })?;
                return Ok(serde_json::json!({
                    "id": id,
                    "status": "deleted"
                }));
            }

            // Also remove vector for project/global deletes
            remove_memory_vector(&e, &id);

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

            embed_memory_entry(&e, &mem.id, &mem.title, &mem.content);

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

/// Return current datetime as RFC 3339 string (UTC, second precision).
fn iso_now() -> String {
    chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string()
}

/// Remove a memory entry's vector from the semantic search store.
fn remove_memory_vector(engine: &EngineState, id: &str) {
    let snapshot = engine.memory_vectors.load_full();
    let mut vectors = (*snapshot).clone();
    vectors.remove(&format!("memory:{}", id));
    engine.memory_vectors.store(Arc::new(vectors));
}

/// Embed a memory entry's content for semantic search.
/// Silently skips if embedder is not loaded (NoopEmbedder).
fn embed_memory_entry(engine: &EngineState, id: &str, title: &str, content: &str) {
    if !engine.embedder.is_loaded() {
        return;
    }
    let text = format!("{} {}", title, content);
    match engine.embedder.embed(&text) {
        Ok(embed_vec) => {
            let snapshot = engine.memory_vectors.load_full();
            let mut vectors: HashMap<String, EmbedVector> = (*snapshot).clone();
            vectors.insert(format!("memory:{}", id), embed_vec);
            engine
                .memory_vectors
                .store(Arc::new(vectors));
        }
        Err(e) => {
            tracing::warn!("Failed to embed memory '{}': {}", id, e);
        }
    }
}
