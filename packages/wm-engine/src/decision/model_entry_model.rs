use serde::Deserialize;

use super::model_file_model::ModelFile;

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct ModelEntry {
    pub name: String,
    pub source: String,
    pub revision: String,
    pub license: String,
    pub max_input_tokens: u32,
    pub files: Vec<ModelFile>,
}
