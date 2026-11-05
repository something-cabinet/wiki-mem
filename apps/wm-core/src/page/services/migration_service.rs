use std::path::{Path, PathBuf};
use std::sync::Arc;
use wm_constants::*;

use super::memory_import_failure_model::MemoryImportFailure;
use super::memory_import_report_model::MemoryImportReport;
use crate::engine::{EngineState, MemoryEntry};
use crate::error::ToolResult;
use crate::page_repo::{FsPageRepo, PageRepo};

const JSON_EXTENSION: &str = "json";
const MEMORY_PAGE_DIR: &str = "memory";

pub fn import_legacy_memory_with_repo(
    engine: &Arc<EngineState>,
    repo: &dyn PageRepo,
) -> ToolResult<MemoryImportReport> {
    let root = engine
        .project_root
        .read()
        .map(|r| r.clone())
        .unwrap_or_else(|_| PathBuf::from("."));
    let old_dir = root.join(WM_DIR).join("memory");
    let mut report = MemoryImportReport::default();

    let Ok(entries) = std::fs::read_dir(&old_dir) else {
        return Ok(report);
    };
    let mut paths: Vec<PathBuf> = entries
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|s| s.to_str()) == Some(JSON_EXTENSION))
        .collect();
    paths.sort();

    for path in paths {
        match import_entry(engine, repo, &path) {
            Ok(()) => report.imported = report.imported.saturating_add(1),
            Err(reason) => report.failed.push(MemoryImportFailure {
                file: file_name_of(&path),
                reason,
            }),
        }
    }

    if report.failed.is_empty() && report.imported > 0 {
        let _ = std::fs::remove_dir(&old_dir);
    }

    tracing::info!(
        "Migrated {} memory entries from JSON to wiki pages ({} failed)",
        report.imported,
        report.failed.len()
    );
    Ok(report)
}

pub fn import_legacy_memory(engine: &Arc<EngineState>) -> ToolResult<MemoryImportReport> {
    import_legacy_memory_with_repo(engine, &FsPageRepo)
}

pub fn migrate_old_memory_json_with_repo(
    engine: &Arc<EngineState>,
    repo: &dyn PageRepo,
) -> ToolResult<usize> {
    import_legacy_memory_with_repo(engine, repo).map(|report| report.imported)
}

pub fn migrate_old_memory_json(engine: &Arc<EngineState>) -> ToolResult<usize> {
    migrate_old_memory_json_with_repo(engine, &FsPageRepo)
}

fn import_entry(engine: &Arc<EngineState>, repo: &dyn PageRepo, path: &Path) -> Result<(), String> {
    let content = std::fs::read_to_string(path).map_err(|error| format!("unreadable: {error}"))?;
    let mem: MemoryEntry =
        serde_json::from_str(&content).map_err(|error| format!("invalid json: {error}"))?;
    let rel_path = format!("{MEMORY_PAGE_DIR}/{}", mem.id);
    let id = crate::page::create_page(engine, &rel_path, &frontmatter_of(&mem), &mem.content)
        .map_err(|error| format!("page not written: {error}"))?;
    let page = crate::page::get_page_with_repo(engine, &id, repo)
        .map_err(|error| format!("page not readable after writing: {error}"))?;
    if !page.raw.contains(&mem.content) {
        return Err("page content differs after writing".to_owned());
    }
    std::fs::remove_file(path).map_err(|error| format!("page written, json not removed: {error}"))
}

fn frontmatter_of(mem: &MemoryEntry) -> String {
    let tags = if mem.tags.is_empty() {
        String::new()
    } else {
        format!("tags: [{}]\n", mem.tags.join(", "))
    };
    let status = mem
        .status
        .as_ref()
        .map(|s| format!("status: {s:?}\n"))
        .unwrap_or_default();
    format!(
        "title: {}\ntype: memory\n{tags}created_at: \"{}\"\nupdated_at: \"{}\"\n{status}",
        serde_json::Value::String(mem.title.clone()),
        mem.created_at,
        mem.updated_at
    )
}

fn file_name_of(path: &Path) -> String {
    path.file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_default()
}
