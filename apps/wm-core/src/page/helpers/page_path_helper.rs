use std::path::Path;
use std::path::PathBuf;
use wm_constants::*;

use crate::error::ToolResult;
use crate::shared::helpers::path_confine_helper::confine;

const WIKI_PREFIX: &str = "wiki/";
const MD_EXT: &str = ".md";

fn wiki_root() -> PathBuf {
    Path::new(WM_DIR).join(WIKI_DIR)
}

fn relative_page_path(path: &str) -> String {
    if path.ends_with(MD_EXT) {
        return path.trim_start_matches(WIKI_PREFIX).to_string();
    }
    let path_part = path.replace(':', "/");
    format!("{}{}", path_part.trim_start_matches(WIKI_PREFIX), MD_EXT)
}

pub fn resolve_page_path(_project_name: &str, path: &str) -> ToolResult<PathBuf> {
    confine(&wiki_root(), Path::new(&relative_page_path(path)))
}

pub fn resolve_id_to_path(_project_root: &Path, id: &str) -> ToolResult<PathBuf> {
    let file_path = resolve_page_path("", id)?;
    if file_path.exists() {
        return Ok(file_path);
    }
    Err(crate::error::ToolError::not_found("page", id))
}

pub fn resolve_simple_page_path(id: &str) -> ToolResult<PathBuf> {
    resolve_page_path("", id)
}
