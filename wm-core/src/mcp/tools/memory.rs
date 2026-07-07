use std::sync::Arc;

use crate::engine::{EngineState, MemoryEntry};
use crate::error::ToolError;
use crate::mcp::handler::ToolArgs;
use crate::mcp::transport::ToolRegistry;

/// Register memory tool handlers
pub fn register(registry: &mut ToolRegistry, engine: Arc<EngineState>) {
    let e = engine.clone();
    registry.register_with_desc(
        "wm_memory.list",
        "List memory entries from .wm/memory/*.json",
        Arc::new(move |params| {
            let args = ToolArgs::new(params);
            let filter_tag = args.optional_string("tag").map(|s| s.to_lowercase());
            let limit = args.optional_int("limit").unwrap_or(50);

            let root = e
                .project_root
                .read()
                .map_err(|_| ToolError::lock_poisoned("project_root"))?
                .clone();

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
}
