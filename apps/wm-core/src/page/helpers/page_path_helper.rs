use std::path::Path;
use std::path::PathBuf;
use wm_constants::*;

use crate::error::{ToolError, ToolResult};

pub fn resolve_page_path(_project_name: &str, path: &str) -> ToolResult<PathBuf> {
    let wiki_dir = Path::new(WM_DIR).join(WIKI_DIR);
    let file_path = if path.ends_with(".md") {
        wiki_dir.join(path.trim_start_matches("wiki/"))
    } else {
        let path_part = path.replace(':', "/");
        wiki_dir.join(format!("{}.md", path_part.trim_start_matches("wiki/")))
    };

    if !file_path.starts_with(&wiki_dir) {
        return Err(ToolError::required_field("path"));
    }

    Ok(file_path)
}

pub fn resolve_id_to_path(_project_root: &Path, id: &str) -> ToolResult<PathBuf> {
    let path_part = id.replace(':', "/");
    let path_part = path_part.strip_prefix("wiki/").unwrap_or(&path_part);
    let file_path = Path::new(WM_DIR)
        .join(WIKI_DIR)
        .join(format!("{}.md", path_part));
    if file_path.exists() {
        return Ok(file_path);
    }
    Err(ToolError::not_found("page", id))
}

pub fn resolve_simple_page_path(id: &str) -> PathBuf {
    let path_part = id.replace(':', "/");
    PathBuf::from(WM_DIR)
        .join(WIKI_DIR)
        .join(format!("{}.md", path_part))
}
