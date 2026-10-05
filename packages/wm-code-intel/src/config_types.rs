use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LspLanguageSettings {
    pub command: String,
    #[serde(default)]
    pub args: Option<Vec<String>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProjectConfigLite {
    #[serde(default)]
    pub lsp: Option<HashMap<String, LspLanguageSettings>>,
}
