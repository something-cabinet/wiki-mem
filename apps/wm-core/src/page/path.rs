use std::path::PathBuf;
use std::path::Path;

use crate::error::{ToolError, ToolResult};

pub fn resolve_page_path(_project_name: &str, path: &str) -> ToolResult<PathBuf> {
    let wiki_dir = Path::new(".wm").join("wiki");
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

pub fn resolve_id_to_path(project_root: &Path, id: &str) -> ToolResult<PathBuf> {
    let path_part = id.replace(':', "/");
    let path_part = path_part.strip_prefix("wiki/").unwrap_or(&path_part);
    let file_path = project_root
        .join(".wm")
        .join("wiki")
        .join(format!("{}.md", path_part));
    if file_path.exists() {
        Ok(file_path)
    } else {
        Err(ToolError::not_found("page", id))
    }
}

pub fn resolve_simple_page_path(id: &str) -> PathBuf {
    let path_part = id.replace(':', "/");
    PathBuf::from(".wm")
        .join("wiki")
        .join(format!("{}.md", path_part))
}
