use std::fmt;
use serde::{Deserialize, Serialize};

/// A code symbol extracted from source.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeIntelSymbol {
    pub name: String,
    pub kind: String,
    pub file: String,
    pub line: usize,
    pub column: usize,
    pub snippet: String,
    pub language: String,
}

impl fmt::Display for CodeIntelSymbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {} [{}:{}] ({})",
            self.kind, self.name, self.file, self.line, self.language
        )
    }
}
