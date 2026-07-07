use serde::{Deserialize, Serialize};
use std::collections::HashMap;
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
        Self {
            preset: "read-write".into(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SearchConfig {
    pub default_mode: String,
    pub default_limit: u32,
    pub rrf_k: u32,
    #[serde(default)]
    pub scoring: ScoringConfig,
}

impl Default for SearchConfig {
    fn default() -> Self {
        Self {
            default_mode: "hybrid".into(),
            default_limit: 20,
            rrf_k: 60,
            scoring: ScoringConfig::default(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScoringConfig {
    #[serde(default = "default_field_weights")]
    pub field_weights: HashMap<String, f64>,
    #[serde(default = "default_recency_model")]
    pub recency_model: String,
    #[serde(default = "default_recency_stability_days")]
    pub recency_stability_days: u32,
    #[serde(default = "default_memory_salience_boost")]
    pub memory_salience_boost: f64,
    #[serde(default = "default_memory_salience_clamp")]
    pub memory_salience_clamp: f64,
    #[serde(default = "default_graph_depth_rrf")]
    pub graph_depth_rrf: u32,
    #[serde(default = "default_graph_depth_retrieve")]
    pub graph_depth_retrieve: u32,
    #[serde(default = "default_graph_depth_retrieve_min_priority")]
    pub graph_depth_retrieve_min_priority: u8,
    #[serde(default = "default_graph_depth_neighbors_default")]
    pub graph_depth_neighbors_default: u32,
    #[serde(default = "default_graph_depth_neighbors_max")]
    pub graph_depth_neighbors_max: u32,
    #[serde(default = "default_debounce_ms")]
    pub debounce_ms: u64,
    #[serde(default = "default_retrieve_token_budget")]
    pub retrieve_token_budget: usize,
}

fn default_field_weights() -> HashMap<String, f64> {
    let mut m = HashMap::new();
    m.insert("title".into(), 4.0);
    m.insert("body".into(), 1.0);
    m
}
fn default_recency_model() -> String { "fsrs".into() }
fn default_recency_stability_days() -> u32 { 7 }
fn default_memory_salience_boost() -> f64 { 2.0 }
fn default_memory_salience_clamp() -> f64 { 0.1 }
fn default_graph_depth_rrf() -> u32 { 1 }
fn default_graph_depth_retrieve() -> u32 { 2 }
fn default_graph_depth_retrieve_min_priority() -> u8 { 5 }
fn default_graph_depth_neighbors_default() -> u32 { 2 }
fn default_graph_depth_neighbors_max() -> u32 { 5 }
fn default_debounce_ms() -> u64 { 500 }
fn default_retrieve_token_budget() -> usize { 2048 }

impl Default for ScoringConfig {
    fn default() -> Self {
        Self {
            field_weights: default_field_weights(),
            recency_model: default_recency_model(),
            recency_stability_days: default_recency_stability_days(),
            memory_salience_boost: default_memory_salience_boost(),
            memory_salience_clamp: default_memory_salience_clamp(),
            graph_depth_rrf: default_graph_depth_rrf(),
            graph_depth_retrieve: default_graph_depth_retrieve(),
            graph_depth_retrieve_min_priority: default_graph_depth_retrieve_min_priority(),
            graph_depth_neighbors_default: default_graph_depth_neighbors_default(),
            graph_depth_neighbors_max: default_graph_depth_neighbors_max(),
            debounce_ms: default_debounce_ms(),
            retrieve_token_budget: default_retrieve_token_budget(),
        }
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
