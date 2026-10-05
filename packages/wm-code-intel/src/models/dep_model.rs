use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeIntelDep {
    pub target: String,
    pub line: usize,
    pub kind: String,
}
