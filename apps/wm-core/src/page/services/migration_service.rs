use std::path::PathBuf;
use std::sync::Arc;

use crate::engine::EngineState;
use wm_error::ToolResult;
use wm_page_repo::{FsPageRepo, PageRepo};

pub fn migrate_old_memory_json_with_repo(engine: &Arc<EngineState>, _repo: &dyn PageRepo) -> ToolResult<usize> {
    let root = engine
        .project_root
        .read()
        .map(|r| r.clone())
        .unwrap_or_else(|_| PathBuf::from("."));
    let old_dir = root.join(".wm").join("memory");

    if !old_dir.exists() {
        return Ok(0);
    }

    let mut migrated = 0usize;

    let entries = match std::fs::read_dir(&old_dir) {
        Ok(e) => e,
        Err(_) => return Ok(0),
    };

    for entry in entries {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("json") {
            continue;
        }

        let content = match std::fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let mem: crate::engine::MemoryEntry = match serde_json::from_str(&content) {
            Ok(m) => m,
            Err(_) => continue,
        };

        let tags_str = if mem.tags.is_empty() {
            String::new()
        } else {
            format!("tags: [{}]\n", mem.tags.join(", "))
        };
        let status_str = mem.status.as_ref().map(|s| format!("status: {:?}\n", s)).unwrap_or_default();
        let frontmatter = format!(
            "title: {}\ntype: memory\n{}created_at: \"{}\"\nupdated_at: \"{}\"\n{}",
            mem.title, tags_str, mem.created_at, mem.updated_at, status_str
        );

        let slug = mem.id;
        let rel_path = format!("memory/{}", slug);
        let _ = crate::page::create_page(engine, &rel_path, &frontmatter, &mem.content);

        let _ = std::fs::remove_file(&path);

        migrated += 1;
    }

    if migrated > 0 {
        let _ = std::fs::remove_dir(&old_dir);
    }

    tracing::info!("Migrated {} memory entries from JSON to wiki pages", migrated);
    Ok(migrated)
}

pub fn migrate_old_memory_json(engine: &Arc<EngineState>) -> ToolResult<usize> {
    migrate_old_memory_json_with_repo(engine, &FsPageRepo)
}
