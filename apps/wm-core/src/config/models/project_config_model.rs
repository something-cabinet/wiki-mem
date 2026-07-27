use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::embedding_config_model::EmbeddingConfig;
use super::git_tracking_model::GitTracking;
use super::lsp_settings_model::LspLanguageSettings;
use super::permissions_config_model::PermissionsConfig;
use super::search_config_model::SearchConfig;
use super::status_colors_model::StatusColors;

#[derive(Debug, Serialize, Deserialize)]
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
