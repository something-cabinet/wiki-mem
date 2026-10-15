use std::path::PathBuf;

use wm_core::config;

pub fn determine_project_root(project: &Option<PathBuf>) -> Result<PathBuf, anyhow::Error> {
    if let Some(path) = project {
        return Ok(path.clone());
    }
    config::detect_project_root()
        .ok_or_else(|| anyhow::anyhow!("No project root found. Run 'wm init' first."))
}
