use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeIndexStats {
    pub files_scanned: usize,
    pub files_changed: usize,
    pub symbols_indexed: usize,
    pub deps_indexed: usize,
    pub edges_indexed: usize,
    pub total_symbols: usize,
    pub total_deps: usize,
    pub total_edges: usize,
    pub errors: Vec<String>,
}
