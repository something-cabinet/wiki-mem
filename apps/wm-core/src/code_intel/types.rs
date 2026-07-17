use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct CodeIntelSymbol {
    pub name: String,
    pub kind: String,
    pub file: String,
    pub line: usize,
    pub column: usize,
    pub snippet: String,
    pub language: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct CodeIntelDep {
    pub target: String,
    pub line: usize,
    pub kind: String,
}
