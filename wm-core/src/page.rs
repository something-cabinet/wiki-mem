use std::path::{Path, PathBuf};
use std::sync::atomic::Ordering;
use std::sync::Arc;

use crate::engine::{EngineState, WikiPageContent};
use crate::error::{ToolError, ToolResult};
use crate::parser::parse_wiki_page;

/// Create a new wiki page
pub fn create_page(
    engine: &Arc<EngineState>,
    path: &str,
    frontmatter: &str,
    content: &str,
) -> ToolResult<String> {
    let full_path = resolve_page_path(&engine.config.read().unwrap().project_name, path)?;

    // Build full markdown
    let full_content = if frontmatter.trim().is_empty() {
        content.to_string()
    } else {
        format!("---\n{}---\n\n{}", frontmatter, content)
    };

    // Write file
    if let Some(parent) = full_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| ToolError::internal(format!("Failed to create directory: {}", e)))?;
    }
    std::fs::write(&full_path, &full_content)
        .map_err(|e| ToolError::internal(format!("Failed to write file: {}", e)))?;

    let meta = parse_wiki_page(&full_path, &full_content);
    engine.stale_flag.store(true, Ordering::Release);

    Ok(meta.id)
}

/// Get a page by its wiki ID (e.g., "wiki:concepts:auth")
pub fn get_page(engine: &Arc<EngineState>, id: &str) -> ToolResult<WikiPageContent> {
    // Look up in the graph snapshot
    let snapshot = engine.graph.load();
    let id_index = &snapshot.1;

    let node_idx = id_index.get(id)
        .ok_or_else(|| ToolError::not_found("page", id))?;

    let _meta = &snapshot.0[*node_idx];
    // Read from disk
    let root = Path::new(".");
    let file_path = resolve_id_to_path(root, id)?;
    let content = std::fs::read_to_string(&file_path)
        .map_err(|e| ToolError::internal(format!("Failed to read {}: {}", file_path.display(), e)))?;

    let sections = crate::parser::split_sections(&content);
    let _meta = parse_wiki_page(&file_path, &content);

    Ok(WikiPageContent {
        raw: content,
        sections: sections.into_iter().map(|(header, body)| {
            let section_id = format!("{}#{}", id, header.to_lowercase().replace(' ', "-"));
            crate::engine::SectionDoc { section_id, page_id: id.to_string(), header, body }
        }).collect(),
    })
}

/// List all page IDs and titles
pub fn list_pages(engine: &Arc<EngineState>) -> ToolResult<Vec<serde_json::Value>> {
    let snapshot = engine.graph.load();
    let graph = &snapshot.0;

    let pages: Vec<serde_json::Value> = graph.node_indices().map(|idx| {
        let meta = &graph[idx];
        serde_json::json!({
            "id": meta.id,
            "title": meta.title,
            "type": format!("{:?}", meta.page_type).to_lowercase(),
            "status": format!("{:?}", meta.status).to_lowercase(),
        })
    }).collect();

    Ok(pages)
}

fn resolve_page_path(_project_name: &str, path: &str) -> ToolResult<PathBuf> {
    // If path has .md, use as-is relative to wiki dir
    let wiki_dir = Path::new(".wm").join("wiki");
    let file_path = if path.ends_with(".md") {
        wiki_dir.join(path.trim_start_matches("wiki/"))
    } else {
        // Generate path from ID: "wiki:concepts:auth" → "wiki/concepts/auth.md"
        let path_part = path.replace(':', "/");
        wiki_dir.join(format!("{}.md", path_part.trim_start_matches("wiki/")))
    };

    // Ensure it's within the wiki directory
    if !file_path.starts_with(&wiki_dir) {
        return Err(ToolError::required_field("path"));
    }

    Ok(file_path)
}

fn resolve_id_to_path(project_root: &Path, id: &str) -> ToolResult<PathBuf> {
    let path_part = id.replace(':', "/");
    let file_path = project_root.join(".wm").join("wiki").join(format!("{}.md", path_part));
    if file_path.exists() {
        Ok(file_path)
    } else {
        Err(ToolError::not_found("page", id))
    }
}
