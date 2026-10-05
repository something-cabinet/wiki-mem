
use std::path::{Path, PathBuf};

use wm_constants::{CODE_DB_FILE, STATE_DIR, WM_DIR};

use crate::code_intel::models::code_index_stats_model::CodeIndexStats;
use crate::code_intel::services::code_index_db::CodeIndexDb;
use crate::code_intel::services::ingest_service::{rebuild_code_index, scan_file_metadata};

const NANOS_PER_SECOND: i64 = 1_000_000_000;

fn code_db_path(project_root: &Path) -> PathBuf {
    project_root.join(WM_DIR).join(STATE_DIR).join(CODE_DB_FILE)
}

pub fn refresh_code_index(project_root: &Path) -> Result<Option<CodeIndexStats>, String> {
    let db_path = code_db_path(project_root);
    if !db_path.exists() {
        return Ok(None);
    }
    let db = CodeIndexDb::open(db_path)?;
    let stats = rebuild_code_index(&db, project_root, false)?;
    Ok(Some(stats))
}

fn indexed_state(db: &CodeIndexDb) -> Result<(usize, i64), String> {
    let hashes = db.load_file_hashes()?;
    let newest = hashes.values().map(|(_, mtime)| *mtime).max().unwrap_or(0);
    Ok((hashes.len(), newest))
}

fn is_stale(db: &CodeIndexDb, project_root: &Path) -> Result<bool, String> {
    let (indexed_files, indexed_mtime) = indexed_state(db)?;
    let (disk_files, disk_mtime) = scan_file_metadata(project_root)?;
    Ok(disk_files != indexed_files || disk_mtime > indexed_mtime)
}

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
