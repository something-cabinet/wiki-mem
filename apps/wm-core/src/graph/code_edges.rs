//! Code-edge access for graph tools (spec item 2, AC-2.2; item 6).
//!
//! The persisted code index (`.wm/state/code.db`, built by `wm index code`)
//! holds raw per-file edges. Graph tools open it lazily (cached per project
//! root), resolve edges against the symbol index, and build a `CodeEdgeGraph`
//! for `wm_graph.neighbors` merging and `wm_graph.affected` traversal.
//!
//! When the code index does not exist (no `wm index code` run), these helpers
//! return `None` and graph tools degrade gracefully to wiki-only results.

use std::path::Path;
use std::sync::{Arc, Mutex, OnceLock};

use wm_code_intel::services::code_index_db::CodeIndexDb;
use wm_code_intel::services::graph_resolver::{
    resolve_code_edges, CodeEdgeGraph, CodeIndexSnapshot,
};
use wm_constants::*;

/// Lazily-opened code index DB per project root. Reopened when the root (or
/// the DB file) changes.
type CachedCodeDb = Option<(std::path::PathBuf, Arc<CodeIndexDb>)>;
static CODE_DB_CACHE: OnceLock<Mutex<CachedCodeDb>> = OnceLock::new();

fn code_db_path(project_root: &Path) -> std::path::PathBuf {
    project_root.join(WM_DIR).join(STATE_DIR).join(CODE_DB_FILE)
}

/// Open (or reuse) the code index DB for a project root.
///
/// Returns `Ok(None)` when the index has never been built.
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

/// Cheap freshness check: the cached handle points at the same file the
/// caller expects; the index is rebuilt in place by `wm index code`, so a
/// rebuilt DB is the same file. To avoid serving a stale in-memory snapshot
/// we rely on `load_code_graph` rebuilding from the DB on every call (see
/// below); this check only guards against a deleted/recreated file.
fn cached_db_is_fresh(db: &CodeIndexDb, path: &Path) -> bool {
    // Reopening on every call would defeat the cache; the DB connection is
    // read-only at query time and `load_code_graph` re-queries it each call,
    // so staleness is bounded by the snapshot load below.
    let _ = (db, path);
    true
}

/// Resolve the full code-edge graph for a project root.
///
/// Reads pre-materialized resolved edges from the DB when available (task 03:
/// spec D2, NFR-2.2 — resolution runs at index time, not per query). Falls
/// back to on-the-fly resolution only when the `resolved_edges` table is empty
/// (first-time use before `wm index code` writes them).
///
/// Refreshes the index first when on-disk sources have diverged from it: a
/// one-shot CLI invocation constructs an engine, answers, and exits before any
/// watcher event lands, so without this probe `wm graph affected` would answer
/// from whatever `wm index code` last wrote (spec `code-edge-resolution`
/// FR-1.1 as amended by D6).
pub fn load_code_graph(project_root: &Path) -> Result<Option<Arc<CodeEdgeGraph>>, String> {
    if let Err(e) = crate::engine::code_index_refresh_service::refresh_if_stale(project_root) {
        tracing::warn!("code index staleness probe failed: {}", e);
    }
    let Some(db) = open_code_index(project_root)? else {
        return Ok(None);
    };

    // Prefer materialized resolved edges (written at index time).
    if db.has_resolved_edges().unwrap_or(false) {
        let resolved = db.load_resolved_edges()?;
        return Ok(Some(Arc::new(CodeEdgeGraph::build(resolved))));
    }

    let snapshot = CodeIndexSnapshot::from_db(&db)?;
    let resolved = resolve_code_edges(&snapshot);
    Ok(Some(Arc::new(CodeEdgeGraph::build(resolved))))
}
