use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub project_name: String,
    pub schema_version: u32,
    #[serde(default)]
    pub embedding: EmbeddingConfig,
    #[serde(default)]
    pub permissions: PermissionsConfig,
    #[serde(default)]
    pub custom_edge_types: Vec<String>,
    #[serde(default)]
    pub source_dirs: Vec<String>,
    #[serde(default)]
    pub source_extensions: Vec<String>,
    #[serde(default)]
    pub search: SearchConfig,
}

impl Default for ProjectConfig {
    fn default() -> Self {
        Self {
            project_name: String::new(),
            schema_version: 1,
            embedding: EmbeddingConfig::default(),
            permissions: PermissionsConfig::default(),
            custom_edge_types: Vec::new(),
            source_dirs: vec!["docs/".into(), "specs/".into()],
            source_extensions: vec!["md".into(), "yaml".into(), "txt".into()],
            search: SearchConfig::default(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EmbeddingConfig {
    pub model_name: String,
    pub dimensions: u32,
    pub batch_size: u32,
}

impl Default for EmbeddingConfig {
    fn default() -> Self {
        Self {
            model_name: "bge-small-en-v1.5".into(),
            dimensions: 384,
            batch_size: 32,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PermissionsConfig {
    pub preset: String,
}

impl Default for PermissionsConfig {
    fn default() -> Self {
        Self { preset: "read-write".into() }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SearchConfig {
    pub default_mode: String,
    pub default_limit: u32,
    pub rrf_k: u32,
}

impl Default for SearchConfig {
    fn default() -> Self {
        Self { default_mode: "hybrid".into(), default_limit: 20, rrf_k: 60 }
    }
}

/// Auto-detect project root by walking up from cwd looking for .wm/config.json
pub fn detect_project_root() -> Option<PathBuf> {
    // Check WM_PROJECT env var first
    if let Ok(path) = std::env::var("WM_PROJECT") {
        let p = PathBuf::from(path);
        if p.join(".wm").join("config.json").exists() {
            return Some(p);
        }
    }

    // Walk up from cwd
    let mut current = std::env::current_dir().ok()?;
    loop {
        if current.join(".wm").join("config.json").exists() {
            return Some(current);
        }
        if !current.pop() {
            break;
        }
    }
    None
}

pub fn load_config(project_root: &Path) -> Result<ProjectConfig, anyhow::Error> {
    let path = project_root.join(".wm").join("config.json");
    let content = std::fs::read_to_string(&path)?;
    let config: ProjectConfig = serde_json::from_str(&content)?;
    Ok(config)
}
