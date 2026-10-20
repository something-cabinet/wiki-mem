use std::path::{Path, PathBuf};
use std::time::Duration;

use wm_constants::{CODE_DB_FILE, CODE_INDEX_STALE_FILE, STATE_DIR, WM_DIR};

use crate::code_intel::models::code_index_stats_model::CodeIndexStats;
use crate::code_intel::services::code_index_db::CodeIndexDb;
use crate::code_intel::services::ingest_service::{rebuild_code_index, scan_file_metadata};

const NANOS_PER_SECOND: i64 = 1_000_000_000;
const REFRESH_ATTEMPTS: u32 = 3;
const REFRESH_BACKOFF: Duration = Duration::from_millis(100);
const BACKOFF_MULTIPLIER: u32 = 4;
const LOCK_CONTENTION_MARKERS: [&str; 2] = ["Locking error", "locked by another process"];

fn code_db_path(project_root: &Path) -> PathBuf {
    project_root.join(WM_DIR).join(STATE_DIR).join(CODE_DB_FILE)
}

fn marker_path(project_root: &Path) -> PathBuf {
    project_root
        .join(WM_DIR)
        .join(STATE_DIR)
        .join(CODE_INDEX_STALE_FILE)
}

pub fn pending_refresh_failure(project_root: &Path) -> Result<Option<String>, String> {
    let path = marker_path(project_root);
    if !path.exists() {
        return Ok(None);
    }
    std::fs::read_to_string(path)
        .map(Some)
        .map_err(|error| format!("read code index stale marker: {}", error))
}

pub fn clear_refresh_failure(project_root: &Path) -> Result<(), String> {
    let path = marker_path(project_root);
    if !path.exists() {
        return Ok(());
    }
    std::fs::remove_file(path).map_err(|error| format!("clear code index stale marker: {}", error))
}

fn record_refresh_failure(project_root: &Path, error: &str) -> Result<(), String> {
    let path = marker_path(project_root);
    let parent = path
        .parent()
        .ok_or_else(|| "code index stale marker has no parent".to_string())?;
    std::fs::create_dir_all(parent)
        .map_err(|cause| format!("create code index stale marker dir: {}", cause))?;
    std::fs::write(path, error).map_err(|cause| format!("write code index stale marker: {}", cause))
}

fn is_lock_contention(error: &str) -> bool {
    LOCK_CONTENTION_MARKERS
        .iter()
        .any(|marker| error.contains(marker))
}

fn retry_refresh<T, F>(attempts: u32, backoff: Duration, mut operation: F) -> Result<T, String>
where
    F: FnMut() -> Result<T, String>,
{
    let mut delay = backoff;
    let mut last_error = String::new();
    for attempt in 1..=attempts {
        match operation() {
            Ok(value) => return Ok(value),
            Err(error) if !is_lock_contention(&error) => return Err(error),
            Err(error) => last_error = error,
        }
        if attempt < attempts {
            std::thread::sleep(delay);
            delay = delay.saturating_mul(BACKOFF_MULTIPLIER);
        }
    }
    Err(last_error)
}

fn clear_marker_after_refresh(project_root: &Path) {
    if let Err(error) = clear_refresh_failure(project_root) {
        tracing::warn!("{}", error);
    }
}

pub fn refresh_code_index_with_retry(
    project_root: &Path,
) -> Result<Option<CodeIndexStats>, String> {
    match retry_refresh(REFRESH_ATTEMPTS, REFRESH_BACKOFF, || {
        refresh_code_index(project_root)
    }) {
        Ok(stats) => Ok(stats),
        Err(error) => {
            record_refresh_failure(project_root, &error)
                .map_err(|marker| format!("{} (stale marker write failed: {})", error, marker))?;
            Err(error)
        }
    }
}

pub fn refresh_code_index(project_root: &Path) -> Result<Option<CodeIndexStats>, String> {
    let db_path = code_db_path(project_root);
    if !db_path.exists() {
        clear_marker_after_refresh(project_root);
        return Ok(None);
    }
    let db = CodeIndexDb::open(db_path)?;
    let stats = rebuild_code_index(&db, project_root, false)?;
    clear_marker_after_refresh(project_root);
    Ok(Some(stats))
}

fn indexed_state(db: &CodeIndexDb) -> Result<(usize, i64), String> {
    let hashes = db.load_file_hashes()?;
    let newest = hashes.values().map(|(_, mtime)| *mtime).max().unwrap_or(0);
    Ok((hashes.len(), newest))
}

fn is_stale(db: &CodeIndexDb, project_root: &Path) -> Result<bool, String> {
    if marker_path(project_root).exists() {
        return Ok(true);
    }
    let (indexed_files, indexed_mtime) = indexed_state(db)?;
    let (disk_files, disk_mtime) = scan_file_metadata(project_root)?;
    Ok(disk_files != indexed_files || disk_mtime > indexed_mtime)
}

pub fn refresh_if_stale(project_root: &Path) -> Result<bool, String> {
    let db_path = code_db_path(project_root);
    if !db_path.exists() {
        clear_marker_after_refresh(project_root);
        return Ok(false);
    }
    let db = CodeIndexDb::open(db_path)?;
    if !is_stale(&db, project_root)? {
        return Ok(false);
    }
    rebuild_code_index(&db, project_root, false)?;
    clear_marker_after_refresh(project_root);
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    const LOCK_ERROR: &str =
        "Locking error: Failed locking file 'code.db-wal'. File is locked by another process";

    #[test]
    fn retry_refresh_stops_after_attempt_budget() {
        let calls = Cell::new(0);

        let result: Result<(), String> = retry_refresh(REFRESH_ATTEMPTS, Duration::ZERO, || {
            calls.set(calls.get() + 1);
            Err(LOCK_ERROR.to_string())
        });

        assert_eq!(result, Err(LOCK_ERROR.to_string()));
        assert_eq!(calls.get(), REFRESH_ATTEMPTS as usize);
    }

    #[test]
    fn retry_refresh_does_not_retry_a_permanent_error() {
        let calls = Cell::new(0);

        let result: Result<(), String> = retry_refresh(REFRESH_ATTEMPTS, Duration::ZERO, || {
            calls.set(calls.get() + 1);
            Err("no such table: symbols".to_string())
        });

        assert_eq!(result, Err("no such table: symbols".to_string()));
        assert_eq!(
            calls.get(),
            1,
            "a non-lock failure must not burn the retry budget"
        );
    }

    #[test]
    fn retry_refresh_returns_first_success() {
        let calls = Cell::new(0);

        let result = retry_refresh(REFRESH_ATTEMPTS, Duration::ZERO, || {
            calls.set(calls.get() + 1);
            if calls.get() == 2 {
                return Ok(7_u32);
            }
            Err(LOCK_ERROR.to_string())
        });

        assert_eq!(result, Ok(7));
        assert_eq!(calls.get(), 2);
    }
}
