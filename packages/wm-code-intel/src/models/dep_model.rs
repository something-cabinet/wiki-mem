use serde::{Deserialize, Serialize};

/// A dependency declaration extracted from source.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeIntelDep {
    pub target: String,
    pub line: usize,
    pub kind: String,
}
