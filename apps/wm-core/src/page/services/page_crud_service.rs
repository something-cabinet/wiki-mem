use std::path::Path;
use std::sync::atomic::Ordering;
use std::sync::Arc;

use crate::engine::{EngineState, PageType, WikiPageContent};
use crate::error::{ToolError, ToolResult};
use crate::page_repo::{FsPageRepo, PageRepo};
use crate::parser::parse_wiki_page;

use crate::page::helpers::page_path_helper::resolve_page_path;

pub fn create_page_with_repo(engine: &Arc<EngineState>, path: &str, frontmatter: &str, content: &str, repo: &dyn PageRepo) -> ToolResult<String> {
    let full_path = resolve_page_path(&engine.config.read().map_err(|_| ToolError::lock_poisoned("config"))?.project_name, path)?;

    let full_content = if frontmatter.trim().is_empty() {
        content.to_string()
    } else {
        format!("---\n{}---\n\n{}", frontmatter, content)
    };

    repo.create_dir_all(full_path.parent().ok_or_else(|| ToolError::internal("invalid path"))?)?;
    repo.write(&full_path, full_content.as_bytes())?;

    // Notify LSP of the new file
    engine.notify_file_changed(&full_path);

    let meta = parse_wiki_page(&full_path, &full_content);
    engine.stale_flag.store(true, Ordering::Release);

    Ok(meta.id)
}

pub fn create_page(engine: &Arc<EngineState>, path: &str, frontmatter: &str, content: &str) -> ToolResult<String> {
    create_page_with_repo(engine, path, frontmatter, content, &FsPageRepo)
}

pub fn get_page_with_repo(engine: &Arc<EngineState>, id: &str, repo: &dyn PageRepo) -> ToolResult<WikiPageContent> {
    let snapshot = engine.graph.load();
    let id_index = &snapshot.1;

    let _node_idx = id_index
        .get(id)
        .ok_or_else(|| ToolError::not_found("page", id))?;

    let root = Path::new(".");
    let file_path = crate::page::helpers::page_path_helper::resolve_id_to_path(root, id)?;
    let content = repo.read_to_string(&file_path)?;

    let sections = crate::parser::parse_sections(&file_path, &content);

    Ok(WikiPageContent {
        raw: content,
        sections,
        meta: None,
    })
}

pub fn get_page(engine: &Arc<EngineState>, id: &str) -> ToolResult<WikiPageContent> {
    get_page_with_repo(engine, id, &FsPageRepo)
}

pub fn get_page_raw_with_repo(engine: &EngineState, id: &str, repo: &dyn PageRepo) -> ToolResult<String> {
    let snapshot = engine.graph.load();
    let index = &snapshot.1;
    let node_idx = index
        .get(id)
        .ok_or_else(|| ToolError::not_found("page", id))?;
    let meta = &snapshot.0[*node_idx];
    let file_path = &meta.path;

    repo.read_to_string(file_path).map_err(|e| {
        ToolError::internal(format!("Failed to read {}: {}", file_path.display(), e))
    })
}

pub fn get_page_raw(engine: &EngineState, id: &str) -> ToolResult<String> {
    get_page_raw_with_repo(engine, id, &FsPageRepo)
}

pub fn list_pages(engine: &Arc<EngineState>, page_type_filter: Option<&PageType>) -> ToolResult<Vec<serde_json::Value>> {
    let snapshot = engine.graph.load();
    let graph = &snapshot.0;

    let pages: Vec<serde_json::Value> = graph
        .node_indices()
        .filter_map(|idx| {
            let meta = &graph[idx];
            if let Some(pt) = page_type_filter {
                if meta.page_type != *pt {
                    return None;
                }
            }
            Some(serde_json::json!({
                "id": meta.id,
                "title": meta.title,
                "type": meta.page_type.as_str(),
                "status": meta.status.as_str(),
            }))
        })
        .collect();

    Ok(pages)
}

pub fn delete_page_with_repo(engine: &Arc<EngineState>, id: &str, repo: &dyn PageRepo) -> ToolResult<()> {
    let snapshot = engine.graph.load();
    let index = &snapshot.1;
    let node_idx = index
        .get(id)
        .ok_or_else(|| ToolError::not_found("page", id))?;
    let meta = &snapshot.0[*node_idx];
    let file_path = &meta.path;

    if repo.exists(file_path) {
        repo.remove_file(file_path)?;
    }

    engine.stale_flag.store(true, std::sync::atomic::Ordering::Release);

    Ok(())
}

pub fn delete_page(engine: &Arc<EngineState>, id: &str) -> ToolResult<()> {
    delete_page_with_repo(engine, id, &FsPageRepo)
}
