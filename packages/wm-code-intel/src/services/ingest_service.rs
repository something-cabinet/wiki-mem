use std::path::Path;

use rayon::prelude::*;
use sha2::{Digest, Sha256};
use walkdir::WalkDir;
use wm_constants::*;

use crate::models::code_index_stats_model::CodeIndexStats;
use crate::services::code_index_db::{CodeIndexDb, FileData};
use crate::{extract_deps, extract_symbols, CodeIntelEngine};

/// Directories to skip during filesystem walking.
const SKIP_DIRS: &[&str] = &[".claude", ".opencode", ".vscode", ".idea"];

fn is_skipped_dir(name: &str) -> bool {
    SKIP_DIRS.contains(&name)
        || wm_constants::SKIP_DIRS.contains(&name)
        || SKIP_DIRS_CODE.contains(&name)
}

/// Rebuild the code index by walking the filesystem from `project_root`.
///
/// For each supported source file:
/// 1. If `force` is false, check mtime against the DB cache — if unchanged, skip hash read.
/// 2. If `force` is false, compute SHA-256 — if unchanged (hash + mtime), skip.
/// 3. If changed/new, parse with tree-sitter.
/// 4. Bulk-upsert all changed files in a single transaction.
/// 5. Delete stale entries for files that no longer exist.
///
/// Returns `CodeIndexStats`: deltas (`*_indexed`) for this run and totals
/// (`total_*`) queried from the DB after upsert and stale cleanup.
///
pub fn rebuild_code_index(
    db: &CodeIndexDb,
    project_root: &Path,
    force: bool,
) -> Result<CodeIndexStats, String> {
    let existing = db.load_file_hashes()?;

    let engine = CodeIntelEngine::global();
    let mut relative_infos: Vec<(String, String)> = Vec::new();
    let mut all_relative_paths: Vec<String> = Vec::new();

    for entry in WalkDir::new(project_root)
        .into_iter()
        .filter_entry(|e| {
            e.file_name()
                .to_str()
                .map(|s| !is_skipped_dir(s))
                .unwrap_or(false)
        })
        .filter_map(|e| e.ok())
    {
        if !entry.file_type().is_file() {
            continue;
        }
        let ext = entry
            .path()
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");
        if engine.is_supported(ext) {
            let rel_path = entry
                .path()
                .strip_prefix(project_root)
                .unwrap_or(entry.path())
                .to_string_lossy()
                .to_string();
            all_relative_paths.push(rel_path.clone());
            relative_infos.push((rel_path, ext.to_string()));
        }
    }

    let changed_data: Vec<FileData> = relative_infos
        .par_iter()
        .filter_map(|(rel_path, ext)| {
            let language = engine.infer_language_from_ext(ext).unwrap_or("unknown");
            let abs_path = project_root.join(rel_path);

            let mtime = match std::fs::metadata(&abs_path) {
                Ok(meta) => meta
                    .modified()
                    .ok()
                    .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                    .map(|d| d.as_nanos() as i64)
                    .unwrap_or(0),
                Err(e) => {
                    tracing::warn!("ingest_service: metadata failed for {}: {}", rel_path, e);
                    return None;
                }
            };

            if !force {
                if let Some((existing_hash, existing_mtime)) = existing.get(rel_path) {
                    if *existing_mtime == mtime {
                        return None;
                    }
                    let content = match std::fs::read_to_string(&abs_path) {
                        Ok(c) => c,
                        Err(e) => {
                            tracing::warn!("ingest_service: failed to read {}: {}", rel_path, e);
                            return None;
                        }
                    };
                    let sha256 = hex::encode(Sha256::digest(content.as_bytes()));
                    if existing_hash == &sha256 {
                        return None;
                    }
                }
            }

            let content = match std::fs::read_to_string(&abs_path) {
                Ok(c) => c,
                Err(e) => {
                    tracing::warn!("ingest_service: failed to read {}: {}", rel_path, e);
                    return None;
                }
            };

            let sha256 = hex::encode(Sha256::digest(content.as_bytes()));

            let syms = extract_symbols(&content, rel_path, ext);
            let deps = extract_deps(&content, ext);

            Some(FileData {
                path: rel_path.clone(),
                sha256,
                mtime,
                language: language.to_string(),
                symbols: syms,
                deps,
            })
        })
        .collect();

    if !changed_data.is_empty() {
        db.bulk_upsert_files(&changed_data)?;
    }

    db.delete_stale_files(&all_relative_paths)?;

    Ok(CodeIndexStats {
        files_scanned: all_relative_paths.len(),
        files_changed: changed_data.len(),
        symbols_indexed: changed_data.iter().map(|f| f.symbols.len()).sum(),
        deps_indexed: changed_data.iter().map(|f| f.deps.len()).sum(),
        total_symbols: db.count_symbols()?,
        total_deps: db.count_deps()?,
        errors: Vec::new(),
    })
}

/// Quick stat-only scan of the filesystem.
///
/// Walks the project root counting supported source files and tracking the
/// maximum modification time. Does NOT read file contents — just metadata.
///
/// Returns `(file_count, max_mtime)`.
///
pub fn scan_file_metadata(project_root: &Path) -> Result<(usize, i64), String> {
    let engine = CodeIntelEngine::global();
    let mut file_count: usize = 0;
    let mut max_mtime: i64 = 0;

    for entry in WalkDir::new(project_root)
        .into_iter()
        .filter_entry(|e| {
            e.file_name()
                .to_str()
                .map(|s| !is_skipped_dir(s))
                .unwrap_or(false)
        })
        .filter_map(|e| e.ok())
    {
        if !entry.file_type().is_file() {
            continue;
        }
        let ext = entry
            .path()
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");
        if engine.is_supported(ext) {
            file_count = file_count.wrapping_add(1);
            if let Ok(meta) = entry.path().metadata() {
                if let Ok(modified) = meta.modified() {
                    if let Ok(duration) = modified.duration_since(std::time::UNIX_EPOCH) {
                        let mtime = duration.as_nanos() as i64;
                        if mtime > max_mtime {
                            max_mtime = mtime;
                        }
                    }
                }
            }
        }
    }

    Ok((file_count, max_mtime))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    fn create_test_file(
        dir: &std::path::Path,
        rel_path: &str,
        content: &str,
    ) -> std::path::PathBuf {
        let full_path = dir.join(rel_path);
        fs::create_dir_all(full_path.parent().unwrap()).unwrap();
        fs::write(&full_path, content).unwrap();
        full_path
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_rebuild_empty_project() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("code.db");
        let db = CodeIndexDb::open(db_path).unwrap();

        let stats = rebuild_code_index(&db, dir.path(), false).unwrap();
        assert_eq!(stats.files_scanned, 0, "no files in empty project");
        assert_eq!(stats.symbols_indexed, 0);
        assert_eq!(stats.deps_indexed, 0);
        assert_eq!(stats.total_symbols, 0);
        assert_eq!(stats.total_deps, 0);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_rebuild_rust_file() {
        let dir = tempdir().unwrap();
        create_test_file(
            dir.path(),
            "src/lib.rs",
            r#"
pub fn hello() -> String { "hello".into() }
pub struct User { name: String }
pub enum Status { Active, Inactive }
"#,
        );

        let db = CodeIndexDb::open(dir.path().join("code.db")).unwrap();
        let stats = rebuild_code_index(&db, dir.path(), false).unwrap();
        assert_eq!(stats.files_scanned, 1, "should find 1 file");
        assert_eq!(stats.symbols_indexed, 3, "hello, User, Status");
        assert_eq!(stats.deps_indexed, 0);
        assert_eq!(stats.total_symbols, 3);
        assert_eq!(stats.total_deps, 0);

        let results = db
            .query_symbols(None, None, None, None, None, None)
            .unwrap();
        assert_eq!(results.len(), 3);
        let names: Vec<&str> = results.iter().map(|s| s.name.as_str()).collect();
        assert!(names.contains(&"hello"));
        assert!(names.contains(&"User"));
        assert!(names.contains(&"Status"));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_rebuild_hash_skip_unchanged() {
        let dir = tempdir().unwrap();
        create_test_file(dir.path(), "src/lib.rs", "pub fn same() -> u32 { 1 }");

        let db = CodeIndexDb::open(dir.path().join("code.db")).unwrap();
        let first = rebuild_code_index(&db, dir.path(), false).unwrap();
        assert_eq!(first.files_scanned, 1);
        assert_eq!(first.symbols_indexed, 1, "first build indexes 1 symbol");

        let second = rebuild_code_index(&db, dir.path(), false).unwrap();
        assert_eq!(second.files_scanned, 1, "still 1 file total");
        assert_eq!(second.symbols_indexed, 0, "0 newly indexed (hash-skip)");
        assert_eq!(second.total_symbols, 1, "total still reflects DB state");
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_rebuild_incremental_new_file() {
        let dir = tempdir().unwrap();
        let src = dir.path().join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(src.join("a.rs"), "pub fn a() {}").unwrap();

        let db = CodeIndexDb::open(dir.path().join("code.db")).unwrap();
        rebuild_code_index(&db, dir.path(), false).unwrap();

        fs::write(src.join("b.rs"), "pub fn b() {}").unwrap();
        let stats = rebuild_code_index(&db, dir.path(), false).unwrap();

        assert_eq!(stats.files_scanned, 2, "total files should be 2");
        assert_eq!(stats.symbols_indexed, 1, "1 newly indexed (the new file)");
        assert_eq!(stats.total_symbols, 2, "totals include both files");
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_rebuild_incremental_edited_file() {
        let dir = tempdir().unwrap();
        let src = dir.path().join("src");
        fs::create_dir_all(&src).unwrap();
        let file_path = src.join("lib.rs");
        fs::write(&file_path, "pub fn original() {}").unwrap();

        let db = CodeIndexDb::open(dir.path().join("code.db")).unwrap();
        rebuild_code_index(&db, dir.path(), false).unwrap();

        fs::write(&file_path, "pub fn original() {}\npub fn added() {}").unwrap();
        let stats = rebuild_code_index(&db, dir.path(), false).unwrap();

        assert_eq!(stats.files_scanned, 1, "still 1 file");
        assert_eq!(
            stats.symbols_indexed, 2,
            "both original and added symbols re-indexed"
        );
        assert_eq!(stats.total_symbols, 2, "totals reflect re-indexed file");

        let results = db
            .query_symbols(None, None, None, None, None, None)
            .unwrap();
        assert_eq!(results.len(), 2);
        let names: Vec<&str> = results.iter().map(|s| s.name.as_str()).collect();
        assert!(names.contains(&"original"));
        assert!(names.contains(&"added"));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_rebuild_incremental_deleted_file() {
        let dir = tempdir().unwrap();
        let src = dir.path().join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(src.join("keep.rs"), "pub fn keep() {}").unwrap();
        fs::write(src.join("gone.rs"), "pub fn gone() {}").unwrap();

        let db = CodeIndexDb::open(dir.path().join("code.db")).unwrap();
        let first = rebuild_code_index(&db, dir.path(), false).unwrap();
        assert_eq!(first.files_scanned, 2);
        assert_eq!(first.symbols_indexed, 2);

        fs::remove_file(src.join("gone.rs")).unwrap();
        let second = rebuild_code_index(&db, dir.path(), false).unwrap();

        assert_eq!(second.files_scanned, 1, "only keep.rs remains");
        assert_eq!(
            second.symbols_indexed, 0,
            "hash-skip: keep.rs unchanged; gone.rs stale-deleted"
        );
        assert_eq!(second.total_symbols, 1, "stale symbols purged");

        let results = db
            .query_symbols(None, None, None, None, None, None)
            .unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "keep");
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_rebuild_force_reparses_unchanged() {
        let dir = tempdir().unwrap();
        create_test_file(dir.path(), "src/lib.rs", "pub fn f() {}");

        let db = CodeIndexDb::open(dir.path().join("code.db")).unwrap();
        let first = rebuild_code_index(&db, dir.path(), false).unwrap();
        assert_eq!(first.total_symbols, 1);
        assert_eq!(first.symbols_indexed, 1);

        let forced = rebuild_code_index(&db, dir.path(), true).unwrap();
        assert_eq!(forced.files_changed, 1, "force re-parses unchanged file");
        assert_eq!(forced.symbols_indexed, 1, "force re-indexes symbols");
        assert_eq!(forced.total_symbols, 1, "totals unchanged after re-parse");
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_rebuild_totals_after_no_change() {
        let dir = tempdir().unwrap();
        create_test_file(
            dir.path(),
            "src/a.rs",
            "use std::collections::HashMap;\npub fn a() {}",
        );
        create_test_file(
            dir.path(),
            "src/b.rs",
            "use std::collections::HashSet;\npub fn b() {}",
        );

        let db = CodeIndexDb::open(dir.path().join("code.db")).unwrap();
        rebuild_code_index(&db, dir.path(), false).unwrap();

        let stats = rebuild_code_index(&db, dir.path(), false).unwrap();
        assert_eq!(stats.symbols_indexed, 0, "delta is 0 on no-change run");
        assert_eq!(stats.deps_indexed, 0, "delta deps is 0 on no-change run");
        assert_eq!(stats.total_symbols, 2, "totals reflect full DB state");
        assert_eq!(stats.total_deps, 2, "totals reflect full DB state");
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_scan_file_metadata() {
        let dir = tempdir().unwrap();
        let src = dir.path().join("src");
        fs::create_dir_all(&src).unwrap();

        fs::write(src.join("lib.rs"), "pub fn f() {}").unwrap();
        fs::write(dir.path().join("README.md"), "# readme").unwrap();

        let (count, _mtime) = scan_file_metadata(dir.path()).unwrap();
        assert_eq!(count, 1, "only .rs file should be counted");
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_skip_dirs_are_skipped() {
        let dir = tempdir().unwrap();
        for skip_dir in &[".git", "node_modules", "target"] {
            let p = dir.path().join(skip_dir).join("lib.rs");
            fs::create_dir_all(p.parent().unwrap()).unwrap();
            fs::write(&p, "pub fn f() {}").unwrap();
        }

        let db = CodeIndexDb::open(dir.path().join("code.db")).unwrap();
        let stats = rebuild_code_index(&db, dir.path(), false).unwrap();
        assert_eq!(
            stats.files_scanned, 0,
            "all files are in skipped directories"
        );
    }
}
