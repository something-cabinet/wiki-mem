use serde::{Deserialize, Serialize};

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
