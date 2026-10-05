use serde::{Deserialize, Serialize};
pub use wm_engine::models::edge_type_model::EdgeProvenance;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CodeEdge {
    pub edge_type: String,
    pub source_file: String,
    pub source_symbol: Option<String>,
    pub target_file: String,
    pub target_symbol: Option<String>,
    pub receiver: Option<String>,
    pub line: usize,
    pub provenance: EdgeProvenance,
}

impl CodeEdge {
    pub fn is_break_sensitive(&self) -> bool {
        matches!(self.edge_type.as_str(), "calls" | "imports" | "inherits")
    }
}
