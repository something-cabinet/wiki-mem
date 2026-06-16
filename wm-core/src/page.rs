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

/// Update an existing wiki page — merge new frontmatter fields
pub fn update_page(engine: &Arc<EngineState>, id: &str, updates: &serde_json::Value) -> ToolResult<()> {
    // Find the page path from graph
    let snapshot = engine.graph.load();
    let index = &snapshot.1;
    let node_idx = index.get(id).ok_or_else(|| ToolError::not_found("page", id))?;
    let meta = &snapshot.0[*node_idx];

    let file_path = &meta.path;
    if !file_path.exists() {
        return Err(ToolError::not_found("page", id));
    }

    let content = std::fs::read_to_string(file_path)
        .map_err(|e| ToolError::internal(format!("Failed to read {}: {}", file_path.display(), e)))?;

    // Parse existing frontmatter
    let (existing_fm, body) = crate::parser::extract_frontmatter(&content);

    // Build updated frontmatter YAML
    let mut new_fm = String::new();

    if let Some(ref fm) = existing_fm {
        if let Some(ref title) = fm.title { new_fm.push_str(&format!("title: {}\n", title)); }
        if let Some(ref pt) = fm.page_type { new_fm.push_str(&format!("type: {}\n", pt)); }
        if !fm.tags.is_empty() { new_fm.push_str(&format!("tags: [{}]\n", fm.tags.join(", "))); }
        if let Some(ref s) = fm.status { new_fm.push_str(&format!("status: {}\n", s)); }
        if let Some(ref p) = fm.priority { new_fm.push_str(&format!("priority: {}\n", p)); }
        if let Some(ref c) = fm.confidence { new_fm.push_str(&format!("confidence: {}\n", c)); }
        if let Some(ref a) = fm.assignee { new_fm.push_str(&format!("assignee: {}\n", a)); }
        if !fm.relates_to.is_empty() {
            new_fm.push_str("relates_to:\n");
            for r in &fm.relates_to {
                new_fm.push_str(&format!("  - {{type: {}, target: {}}}\n", r.edge_type, r.target));
            }
        }
        if !fm.acceptance_criteria.is_empty() {
            new_fm.push_str("acceptance_criteria:\n");
            for ac in &fm.acceptance_criteria {
                new_fm.push_str(&format!("  - {{text: \"{}\", checked: {}}}\n", ac.text, ac.checked));
            }
        }
    }

    // Override with update fields
    if let Some(title) = updates.get("title").and_then(|v| v.as_str()) {
        new_fm = set_yaml_field(&new_fm, "title", title);
    }
    if let Some(status) = updates.get("status").and_then(|v| v.as_str()) {
        new_fm = set_yaml_field(&new_fm, "status", status);
    }

    // Handle checked_ac / unchecked_ac
    if let Some(check_list) = updates.get("checked_ac").and_then(|v| v.as_array()) {
        for idx in check_list.iter().filter_map(|v| v.as_u64()) {
            new_fm = ac_set_checked(&new_fm, idx as usize, true);
        }
    }
    if let Some(uncheck_list) = updates.get("unchecked_ac").and_then(|v| v.as_array()) {
        for idx in uncheck_list.iter().filter_map(|v| v.as_u64()) {
            new_fm = ac_set_checked(&new_fm, idx as usize, false);
        }
    }

    let full = format!("---\n{}---\n\n{}", new_fm, body);
    std::fs::write(file_path, &full)
        .map_err(|e| ToolError::internal(format!("Failed to write {}: {}", file_path.display(), e)))?;

    engine.stale_flag.store(true, Ordering::Release);
    Ok(())
}

fn set_yaml_field(yaml: &str, key: &str, value: &str) -> String {
    let mut found = false;
    let mut result = String::new();
    for line in yaml.lines() {
        if line.starts_with(&format!("{}:", key)) {
            result.push_str(&format!("{}: {}\n", key, value));
            found = true;
        } else {
            result.push_str(line);
            result.push('\n');
        }
    }
    if !found {
        result.push_str(&format!("{}: {}\n", key, value));
    }
    result
}

fn ac_set_checked(yaml: &str, index: usize, checked: bool) -> String {
    let mut current_ac = 0usize;
    let mut result = String::new();
    for line in yaml.lines() {
        if line.trim_start().starts_with("- {text:") {
            current_ac += 1;
            if current_ac == index {
                if let Some(pos) = line.find("checked:") {
                    let end = pos + 8;
                    let before = &line[..end];
                    let after = &line[line.len().saturating_sub(1)..];
                    result.push_str(&format!("{}{}{}\n", before, checked, after));
                    continue;
                }
            }
        }
        result.push_str(line);
        result.push('\n');
    }
    result
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
