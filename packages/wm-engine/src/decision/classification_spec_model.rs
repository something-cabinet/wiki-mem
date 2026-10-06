use super::classification_mode_model::ClassificationMode;

#[derive(Clone, Debug, PartialEq)]
pub struct ClassificationSpec {
    pub task: String,
    pub prompt: String,
    pub labels: Vec<String>,
    pub mode: ClassificationMode,
    pub label_descriptions: Vec<(String, String)>,
    pub examples: Vec<(String, String)>,
}
