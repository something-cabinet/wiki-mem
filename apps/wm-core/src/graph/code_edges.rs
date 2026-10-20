use std::path::Path;
use std::sync::Arc;

use wm_code_intel::services::code_index_db::CodeIndexDb;
use wm_code_intel::services::graph_resolver::{
    resolve_code_edges, CodeEdgeGraph, CodeIndexSnapshot, ResolvedCodeEdge,
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
    if let Err(error) = crate::engine::code_index_refresh_service::refresh_if_stale(project_root) {
        tracing::warn!(
            "code index refresh failed; serving filesystem snapshot: {}",
            error
        );
        return filesystem_graph(project_root);
    }
    match open_code_index(project_root) {
        Ok(None) => Ok(None),
        Ok(Some(db)) => graph_from_db(project_root, &db),
        Err(error) => {
            tracing::warn!(
                "code index open failed; serving filesystem snapshot: {}",
                error
            );
            filesystem_graph(project_root)
        }
    }
}

fn graph_from_db(
    project_root: &Path,
    db: &CodeIndexDb,
) -> Result<Option<Arc<CodeEdgeGraph>>, String> {
    match graph_payload(db) {
        Ok(resolved) => Ok(Some(Arc::new(CodeEdgeGraph::build(resolved)))),
        Err(error) => {
            tracing::warn!(
                "code index read failed; serving filesystem snapshot: {}",
                error
            );
            filesystem_graph(project_root)
        }
    }
}

fn graph_payload(db: &CodeIndexDb) -> Result<Vec<ResolvedCodeEdge>, String> {
    match db.has_resolved_edges()? {
        true => db.load_resolved_edges(),
        false => {
            let snapshot = CodeIndexSnapshot::from_db(db)?;
            Ok(resolve_code_edges(&snapshot))
        }
    }
}

fn filesystem_graph(project_root: &Path) -> Result<Option<Arc<CodeEdgeGraph>>, String> {
    let snapshot = CodeIndexSnapshot::collect_from_fs(project_root)?;
    let resolved = resolve_code_edges(&snapshot);
    Ok(Some(Arc::new(CodeEdgeGraph::build(resolved))))
}
