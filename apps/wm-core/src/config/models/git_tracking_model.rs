use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use wm_constants::*;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct GitTracking {
    #[serde(default)]
    pub memory: Option<bool>,
    #[serde(default)]
    pub versions: Option<bool>,
    #[serde(default)]
    pub state: Option<bool>,
}

pub fn apply_git_tracking(root: &Path, tracking: &GitTracking) -> Result<usize, std::io::Error> {
    let gitignore_path = root.join(".gitignore");
    let mut content = if gitignore_path.exists() {
        std::fs::read_to_string(&gitignore_path).unwrap_or_default()
    } else {
        String::new()
    };
    let mut modified = 0usize;

    let entries: [(&str, Option<bool>); 3] = [
        (".wm/memory/", tracking.memory),
        (".wm/state/", tracking.state),
        (".wm/versions/", tracking.versions),
    ];

    for &(entry, enabled) in &entries {
        if enabled == Some(true) && !content.contains(entry) {
            let line = format!("# Wiki Memory Engine\n{}\n", entry);
            content.push_str(&line);
            modified = modified.wrapping_add(1);
        } else if enabled == Some(false) {
            let lines: Vec<String> = content
                .lines()
                .filter(|l| !l.contains(entry))
                .map(|l| l.to_string())
                .collect();
            let new_content = lines.join("\n");
            if new_content.len() < content.len() {
                content = new_content;
                modified = modified.wrapping_add(1);
            }
        }
    }

    if modified > 0 {
        std::fs::write(&gitignore_path, &content)?;
    }

    Ok(modified)
}

pub fn detect_project_root() -> Option<PathBuf> {
    if let Ok(path) = std::env::var("WM_PROJECT") {
        let p = PathBuf::from(path);
        if p.join(WM_DIR).join(CONFIG_FILE).exists() {
            return Some(p);
        }
    }

    let mut current = std::env::current_dir().ok()?;
    loop {
        if current.join(WM_DIR).join(CONFIG_FILE).exists() {
            return Some(current);
        }
        if !current.pop() {
            break;
        }
    }
    None
}

pub fn load_config(
    project_root: &Path,
) -> Result<super::project_config_model::ProjectConfig, anyhow::Error> {
    let path = project_root.join(WM_DIR).join(CONFIG_FILE);
    let content = std::fs::read_to_string(&path)?;
    let config: super::project_config_model::ProjectConfig = serde_json::from_str(&content)?;
    Ok(config)
}
