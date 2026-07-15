use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::embed::SearchMode;

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
    #[serde(default)]
    pub status_colors: StatusColors,
    #[serde(default)]
    pub visible_columns: Option<Vec<String>>,
    #[serde(default)]
    pub lsp: Option<HashMap<String, LspLanguageSettings>>,
    #[serde(default)]
    pub git_tracking: Option<GitTracking>,
    #[serde(default)]
    pub runtime_memory_max_entries: Option<u32>,
    #[serde(default)]
    pub runtime_memory_recency_days: Option<u32>,
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
            status_colors: StatusColors::default(),
            visible_columns: None,
            lsp: None,
            git_tracking: None,
            runtime_memory_max_entries: None,
            runtime_memory_recency_days: None,
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

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum PermissionPreset {
    ReadWrite,
    ReadOnly,
}

impl Default for PermissionPreset {
    fn default() -> Self {
        PermissionPreset::ReadWrite
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PermissionsConfig {
    pub preset: PermissionPreset,
}

impl Default for PermissionsConfig {
    fn default() -> Self {
        Self {
            preset: PermissionPreset::default(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SearchConfig {
    #[serde(default)]
    pub default_mode: SearchMode,
    pub default_limit: u32,
    pub rrf_k: u32,
    #[serde(default)]
    pub scoring: ScoringConfig,
}

impl Default for SearchConfig {
    fn default() -> Self {
        Self {
            default_mode: SearchMode::Hybrid,
            default_limit: 20,
            rrf_k: 60,
            scoring: ScoringConfig::default(),
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum RecencyModel {
    Fsrs,
    Linear,
    Exponential,
    None,
}

impl Default for RecencyModel {
    fn default() -> Self {
        RecencyModel::Fsrs
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScoringConfig {
    #[serde(default = "default_field_weights")]
    pub field_weights: HashMap<String, f64>,
    #[serde(default)]
    pub recency_model: RecencyModel,
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
            recency_model: RecencyModel::default(),
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

// ── Status colors ───────────────────────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StatusColors {
    pub colors: HashMap<String, String>,
}

impl Default for StatusColors {
    fn default() -> Self {
        let mut colors = HashMap::new();
        colors.insert("todo".into(), "gray".into());
        colors.insert("in-progress".into(), "blue".into());
        colors.insert("in-review".into(), "violet".into());
        colors.insert("done".into(), "green".into());
        colors.insert("blocked".into(), "red".into());
        colors.insert("on-hold".into(), "amber".into());
        colors.insert("urgent".into(), "rose".into());
        Self { colors }
    }
}

// ── LSP language settings ────────────────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LspLanguageSettings {
    pub command: String,
    #[serde(default)]
    pub args: Option<Vec<String>>,
}

impl Default for LspLanguageSettings {
    fn default() -> Self {
        Self {
            command: String::new(),
            args: None,
        }
    }
}

// ── Git tracking per-section ─────────────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GitTracking {
    #[serde(default)]
    pub memory: Option<bool>,
    #[serde(default)]
    pub versions: Option<bool>,
    #[serde(default)]
    pub state: Option<bool>,
}

impl Default for GitTracking {
    fn default() -> Self {
        Self {
            memory: None,
            versions: None,
            state: None,
        }
    }
}

/// Apply git tracking settings to the project's `.gitignore` file.
///
/// - When `tracking.memory` is `Some(true)`: the `.wm/memory/` directory is added to `.gitignore`.
/// - When `tracking.state` is `Some(true)`: the `.wm/state/` directory is added to `.gitignore`.
/// - When `tracking.versions` is `Some(true)`: the `.wm/versions/` directory is added to `.gitignore`.
/// - When any field is `Some(false)`: the corresponding entry is *removed* from `.gitignore`.
/// - When a field is `None`: the entry is left as-is (no change).
///
/// Returns the number of entries that were modified.
pub fn apply_git_tracking(root: &Path, tracking: &GitTracking) -> Result<usize, std::io::Error> {
    let gitignore_path = root.join(".gitignore");
    let mut content = if gitignore_path.exists() {
        std::fs::read_to_string(&gitignore_path).unwrap_or_default()
    } else {
        String::new()
    };
    let mut modified = 0usize;

    // Define the per-section entries and their corresponding config fields
    let entries: [(&str, Option<bool>); 3] = [
        (".wm/memory/",   tracking.memory),
        (".wm/state/",    tracking.state),
        (".wm/versions/", tracking.versions),
    ];

    for &(entry, enabled) in &entries {
        match enabled {
            Some(true) => {
                // Add entry if not already present
                if !content.contains(entry) {
                    // Find the section header or add at end
                    let line = format!("# Wiki Memory Engine\n{}\n", entry);
                    content.push_str(&line);
                    modified += 1;
                }
            }
            Some(false) => {
                // Remove entry (and the comment line before it if it's the WME header)
                let lines: Vec<String> = content.lines()
                    .filter(|l| !l.contains(entry))
                    .map(|l| l.to_string())
                    .collect();
                let new_content = lines.join("\n");
                if new_content.len() < content.len() {
                    content = new_content;
                    modified += 1;
                }
            }
            None => { /* leave as-is */ }
        }
    }

    if modified > 0 {
        std::fs::write(&gitignore_path, &content)?;
    }

    Ok(modified)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scoring_config_defaults() {
        let cfg = ScoringConfig::default();
        // Verify all default values match expected defaults
        assert_eq!(cfg.field_weights.get("title"), Some(&4.0));
        assert_eq!(cfg.field_weights.get("body"), Some(&1.0));
        assert_eq!(cfg.recency_model, RecencyModel::Fsrs);
        assert_eq!(cfg.recency_stability_days, 7);
        assert!((cfg.memory_salience_boost - 2.0).abs() < 1e-9);
        assert!((cfg.memory_salience_clamp - 0.1).abs() < 1e-9);
        assert_eq!(cfg.graph_depth_rrf, 1);
        assert_eq!(cfg.graph_depth_retrieve, 2);
        assert_eq!(cfg.graph_depth_retrieve_min_priority, 5);
        assert_eq!(cfg.graph_depth_neighbors_default, 2);
        assert_eq!(cfg.graph_depth_neighbors_max, 5);
        assert_eq!(cfg.debounce_ms, 500);
        assert_eq!(cfg.retrieve_token_budget, 2048);
    }

    #[test]
    fn test_search_config_defaults() {
        let cfg = SearchConfig::default();
        assert_eq!(cfg.default_mode, SearchMode::Hybrid);
        assert_eq!(cfg.default_limit, 20);
        assert_eq!(cfg.rrf_k, 60);
    }

    #[test]
    fn test_embedding_config_defaults() {
        let cfg = EmbeddingConfig::default();
        assert_eq!(cfg.model_name, "bge-small-en-v1.5");
        assert_eq!(cfg.dimensions, 384);
        assert_eq!(cfg.batch_size, 32);
    }

    #[test]
    fn test_permissions_config_defaults() {
        let cfg = PermissionsConfig::default();
        assert_eq!(cfg.preset, PermissionPreset::ReadWrite);
    }

    #[test]
    fn test_project_config_default_deserializes_from_valid_json() {
        // Verify that default() produces valid JSON that can be round-tripped
        let cfg = ProjectConfig::default();
        let json = serde_json::to_string(&cfg).expect("Serialization should succeed");
        let deserialized: ProjectConfig =
            serde_json::from_str(&json).expect("Deserialization should succeed from valid JSON");
        assert_eq!(deserialized.project_name, cfg.project_name);
        assert_eq!(deserialized.schema_version, cfg.schema_version);
        assert_eq!(
            deserialized.embedding.model_name,
            cfg.embedding.model_name
        );
        assert_eq!(deserialized.search.default_mode, cfg.search.default_mode);
        assert_eq!(
            deserialized.search.scoring.recency_model,
            cfg.search.scoring.recency_model
        );
        assert_eq!(
            deserialized.search.scoring.recency_stability_days,
            cfg.search.scoring.recency_stability_days
        );
        assert_eq!(
            deserialized.search.scoring.graph_depth_rrf,
            cfg.search.scoring.graph_depth_rrf
        );
        assert_eq!(
            deserialized.search.scoring.debounce_ms,
            cfg.search.scoring.debounce_ms
        );
    }

    #[test]
    fn test_scoring_config_optional_field_defaults() {
        let json = r#"{"field_weights": {"title": 2.0, "body": 1.0}}"#;
        let config: ScoringConfig = serde_json::from_str(json).expect("should deserialize");
        assert_eq!(config.field_weights["title"], 2.0);
        assert_eq!(config.recency_model, RecencyModel::Fsrs); // default
    }

    #[test]
    fn test_permission_preset_deserialize() {
        assert_eq!(
            serde_json::from_str::<PermissionPreset>("\"read-write\"").unwrap(),
            PermissionPreset::ReadWrite
        );
        assert_eq!(
            serde_json::from_str::<PermissionPreset>("\"read-only\"").unwrap(),
            PermissionPreset::ReadOnly
        );
        assert!(serde_json::from_str::<PermissionPreset>("\"invalid\"").is_err());
    }

    #[test]
    fn test_recency_model_deserialize() {
        assert_eq!(
            serde_json::from_str::<RecencyModel>("\"fsrs\"").unwrap(),
            RecencyModel::Fsrs
        );
        assert_eq!(
            serde_json::from_str::<RecencyModel>("\"linear\"").unwrap(),
            RecencyModel::Linear
        );
        assert_eq!(
            serde_json::from_str::<RecencyModel>("\"exponential\"").unwrap(),
            RecencyModel::Exponential
        );
        assert_eq!(
            serde_json::from_str::<RecencyModel>("\"none\"").unwrap(),
            RecencyModel::None
        );
    }
}
