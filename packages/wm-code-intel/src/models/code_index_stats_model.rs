use serde::{Deserialize, Serialize};

/// Statistics for a single code index rebuild run.
///
/// `files_scanned` and `files_changed` cover the filesystem walk, the
/// `*_indexed` fields are deltas written this run, and `total_*` reflect
/// the full database state after the run completes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeIndexStats {
    pub files_scanned: usize,
    pub files_changed: usize,
    pub symbols_indexed: usize,
    pub deps_indexed: usize,
    pub total_symbols: usize,
    pub total_deps: usize,
    pub errors: Vec<String>,
}
