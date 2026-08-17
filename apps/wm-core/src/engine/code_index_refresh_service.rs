//! Code-index freshness.
//!
//! Three entry points feed one incremental rebuild:
//!
//! - [`refresh_code_index`] is driven by the file watcher
//!   (`main_engine_factory`) for long-lived processes: `wm-cli mcp` sessions
//!   and the `wm-server` daemon.
//! - [`refresh_if_stale`] is the read-time probe for one-shot CLI invocations,
//!   which construct an engine, run, and exit before any watcher event lands.
//!   Without it, `wm graph affected` from the CLI would keep answering from
//!   whatever `wm index code` last wrote.
//! - [`index_lag_seconds`] is the reporting primitive behind staleness; both the
//!   CLI `wm index code` output and `wm_index_status` consume it.
//!
//! Code intelligence stays opt-in: neither entry point creates an index that
//! `wm index code` has never built. Absent index means absent code edges, and
//! graph tools already degrade to wiki-only results in that case.

use std::path::{Path, PathBuf};

use wm_constants::{CODE_DB_FILE, STATE_DIR, WM_DIR};

use crate::code_intel::models::code_index_stats_model::CodeIndexStats;
use crate::code_intel::services::code_index_db::CodeIndexDb;
use crate::code_intel::services::ingest_service::{rebuild_code_index, scan_file_metadata};

const NANOS_PER_SECOND: i64 = 1_000_000_000;

fn code_db_path(project_root: &Path) -> PathBuf {
    project_root.join(WM_DIR).join(STATE_DIR).join(CODE_DB_FILE)
}

/// Incrementally re-extract changed source files into an existing code index.
///
/// Delegates to the content-hash incremental rebuild, so unchanged files are
/// never re-parsed and files that disappeared are pruned.
///
/// Returns `Ok(None)` when no index exists.
pub fn refresh_code_index(project_root: &Path) -> Result<Option<CodeIndexStats>, String> {
    let db_path = code_db_path(project_root);
    if !db_path.exists() {
        return Ok(None);
    }
    let db = CodeIndexDb::open(db_path)?;
    let stats = rebuild_code_index(&db, project_root, false)?;
    Ok(Some(stats))
}

/// Indexed file count and newest indexed mtime, in nanoseconds.
fn indexed_state(db: &CodeIndexDb) -> Result<(usize, i64), String> {
    let hashes = db.load_file_hashes()?;
    let newest = hashes.values().map(|(_, mtime)| *mtime).max().unwrap_or(0);
    Ok((hashes.len(), newest))
}

/// Whether the on-disk source tree diverges from the indexed state.
///
/// A differing supported-file count catches additions and deletions; a newer
/// on-disk mtime catches edits. Metadata only — no file contents are read.
fn is_stale(db: &CodeIndexDb, project_root: &Path) -> Result<bool, String> {
    let (indexed_files, indexed_mtime) = indexed_state(db)?;
    let (disk_files, disk_mtime) = scan_file_metadata(project_root)?;
    Ok(disk_files != indexed_files || disk_mtime > indexed_mtime)
}

/// Refresh the code index when on-disk sources diverge from the indexed state.
///
/// This is the one-shot-invocation counterpart to the watcher, called from the
/// code-graph read path (`graph::code_edges::load_code_graph`). Returns whether
/// a rebuild ran, so callers can report it.
pub fn refresh_if_stale(project_root: &Path) -> Result<bool, String> {
    let db_path = code_db_path(project_root);
    if !db_path.exists() {
        return Ok(false);
    }
    let db = CodeIndexDb::open(db_path)?;
    if !is_stale(&db, project_root)? {
        return Ok(false);
    }
    rebuild_code_index(&db, project_root, false)?;
    Ok(true)
}

/// How far the newest source file is ahead of the newest indexed file, in
/// seconds. `Some(0)` means current; `None` means no index exists.
///
/// This is the primitive behind staleness reporting; the CLI
/// (`wm index code`) and `wm_index_status` consume it.
pub fn index_lag_seconds(project_root: &Path) -> Result<Option<i64>, String> {
    let db_path = code_db_path(project_root);
    if !db_path.exists() {
        return Ok(None);
    }
    let db = CodeIndexDb::open(db_path)?;
    let (_, indexed_mtime) = indexed_state(&db)?;
    let (_, disk_mtime) = scan_file_metadata(project_root)?;
    let lag_nanos = disk_mtime.saturating_sub(indexed_mtime).max(0);
    Ok(Some(lag_nanos / NANOS_PER_SECOND))
}
