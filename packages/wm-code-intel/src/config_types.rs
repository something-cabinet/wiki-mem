use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Minimal LSP language settings.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LspLanguageSettings {
    pub command: String,
    #[serde(default)]
    pub args: Option<Vec<String>>,
}

/// Minimal project config — only the fields needed by code-intel.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProjectConfigLite {
    #[serde(default)]
    pub lsp: Option<HashMap<String, LspLanguageSettings>>,
}
