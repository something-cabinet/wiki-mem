
use std::path::Path;
use std::sync::{Arc, Mutex, OnceLock};

use wm_code_intel::services::code_index_db::CodeIndexDb;
use wm_code_intel::services::graph_resolver::{
    resolve_code_edges, CodeEdgeGraph, CodeIndexSnapshot,
};
use wm_constants::*;

type CachedCodeDb = Option<(std::path::PathBuf, Arc<CodeIndexDb>)>;
static CODE_DB_CACHE: OnceLock<Mutex<CachedCodeDb>> = OnceLock::new();

fn code_db_path(project_root: &Path) -> std::path::PathBuf {
    project_root.join(WM_DIR).join(STATE_DIR).join(CODE_DB_FILE)
}

pub fn open_code_index(project_root: &Path) -> Result<Option<Arc<CodeIndexDb>>, String> {
    let db_path = code_db_path(project_root);
    if !db_path.exists() {
        return Ok(None);
    }
    let cache = CODE_DB_CACHE.get_or_init(|| Mutex::new(None));
    let mut guard = cache
        .lock()
        .map_err(|_| "code db cache poisoned".to_string())?;
    if let Some((cached_root, cached_db)) = guard.as_ref() {
        if cached_root == project_root && cached_db_is_fresh(cached_db, &db_path) {
            return Ok(Some(cached_db.clone()));
        }
    }
    let db = Arc::new(CodeIndexDb::open(db_path)?);
    *guard = Some((project_root.to_path_buf(), db.clone()));
    Ok(Some(db))
}

fn cached_db_is_fresh(db: &CodeIndexDb, path: &Path) -> bool {
    let _ = (db, path);
    true
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
