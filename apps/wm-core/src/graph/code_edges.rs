use std::path::Path;
use std::sync::Arc;

use wm_code_intel::services::code_index_db::CodeIndexDb;
use wm_code_intel::services::graph_resolver::{
    resolve_code_edges, CodeEdgeGraph, CodeIndexSnapshot,
};
use wm_constants::*;

fn code_db_path(project_root: &Path) -> std::path::PathBuf {
    project_root.join(WM_DIR).join(STATE_DIR).join(CODE_DB_FILE)
}

pub fn open_code_index(project_root: &Path) -> Result<Option<CodeIndexDb>, String> {
    let db_path = code_db_path(project_root);
    if !db_path.exists() {
        return Ok(None);
    }
    Ok(Some(CodeIndexDb::open(db_path)?))
}

pub fn load_code_graph(project_root: &Path) -> Result<Option<Arc<CodeEdgeGraph>>, String> {
    if let Err(e) = crate::engine::code_index_refresh_service::refresh_if_stale(project_root) {
        tracing::warn!("code index staleness probe failed: {}", e);
    }
    let Some(db) = open_code_index(project_root)? else {
        return Ok(None);
    };

    if db.has_resolved_edges().unwrap_or(false) {
        let resolved = db.load_resolved_edges()?;
        return Ok(Some(Arc::new(CodeEdgeGraph::build(resolved))));
    }

    let snapshot = CodeIndexSnapshot::from_db(&db)?;
    let resolved = resolve_code_edges(&snapshot);
    Ok(Some(Arc::new(CodeEdgeGraph::build(resolved))))
}
