use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct LspLanguageSettings {
    pub command: String,
    #[serde(default)]
    pub args: Option<Vec<String>>,
}
