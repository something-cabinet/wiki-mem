use std::collections::{HashMap, HashSet};
use std::path::Path;

use rayon::prelude::*;
use sha2::{Digest, Sha256};

use crate::models::code_index_stats_model::CodeIndexStats;
use crate::services::code_index_db::{CodeIndexDb, FileData};
use crate::services::graph_resolver::{resolve_code_edges, CodeIndexSnapshot};
use crate::services::ignore_policy_service::code_walk;
use crate::{extract_deps, extract_edges, extract_symbols, CodeIntelEngine};

struct ProjectScan {
    all_paths: Vec<String>,
    changed: Vec<FileData>,
}

#[derive(Default)]
struct ProjectMetadata {
    count: usize,
    max_mtime: i64,
}

pub fn rebuild_code_index(
    db_path: &Path,
    project_root: &Path,
    force: bool,
) -> Result<CodeIndexStats, String> {
    ensure_db_dir(db_path)?;
    let existing = load_existing_hashes(db_path)?;
    let scan = scan_project(project_root, &existing, force);
    let db = CodeIndexDb::open(db_path.to_path_buf())?;
    apply_scan(&db, project_root, &existing, &scan)?;
    code_index_stats(&db, &scan)
}

fn ensure_db_dir(db_path: &Path) -> Result<(), String> {
    let Some(parent) = db_path.parent() else {
        return Ok(());
    };
    if parent.as_os_str().is_empty() {
        return Ok(());
    }
    std::fs::create_dir_all(parent).map_err(|error| format!("create code index dir: {}", error))
}

fn load_existing_hashes(db_path: &Path) -> Result<HashMap<String, (String, i64)>, String> {
    let db = CodeIndexDb::open(db_path.to_path_buf())?;
    db.load_file_hashes()
}

fn source_extension<'a>(engine: &CodeIntelEngine, path: &'a Path) -> Option<&'a str> {
    let ext = path.extension().and_then(|e| e.to_str())?;
    engine.is_supported(ext).then_some(ext)
}

fn relative_path(project_root: &Path, path: &Path) -> String {
    path.strip_prefix(project_root)
        .unwrap_or(path)
        .to_string_lossy()
        .to_string()
}

fn list_supported_files(project_root: &Path) -> (Vec<String>, Vec<(String, String)>) {
    let engine = CodeIntelEngine::global();
    let mut all_paths = Vec::new();
    let mut infos = Vec::new();
    for entry in code_walk(project_root).filter_map(Result::ok) {
        if !entry.file_type().is_some_and(|t| t.is_file()) {
            continue;
        }
        let Some(ext) = source_extension(engine, entry.path()) else {
            continue;
        };
        let rel_path = relative_path(project_root, entry.path());
        all_paths.push(rel_path.clone());
        infos.push((rel_path, ext.to_string()));
    }
    (all_paths, infos)
}

fn entry_mtime(path: &Path) -> i64 {
    path.metadata()
        .ok()
        .and_then(|meta| meta.modified().ok())
        .and_then(|modified| modified.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|elapsed| elapsed.as_nanos() as i64)
        .unwrap_or(0)
}

fn file_mtime(abs_path: &Path, rel_path: &str) -> Option<i64> {
    let meta = match std::fs::metadata(abs_path) {
        Ok(meta) => meta,
        Err(error) => {
            tracing::warn!(
                "ingest_service: metadata failed for {}: {}",
                rel_path,
                error
            );
            return None;
        }
    };
    let modified = meta.modified().ok()?;
    let elapsed = modified.duration_since(std::time::UNIX_EPOCH).ok()?;
    Some(elapsed.as_nanos() as i64)
}

fn read_source(abs_path: &Path, rel_path: &str) -> Option<String> {
    match std::fs::read_to_string(abs_path) {
        Ok(content) => Some(content),
        Err(error) => {
            tracing::warn!("ingest_service: failed to read {}: {}", rel_path, error);
            None
        }
    }
}

fn unchanged_by_mtime(
    existing: &HashMap<String, (String, i64)>,
    rel_path: &str,
    mtime: i64,
) -> bool {
    existing
        .get(rel_path)
        .is_some_and(|(_, indexed_mtime)| *indexed_mtime == mtime)
}

fn unchanged_by_hash(
    existing: &HashMap<String, (String, i64)>,
    rel_path: &str,
    sha256: &str,
) -> bool {
    existing
        .get(rel_path)
        .is_some_and(|(indexed_hash, _)| indexed_hash == sha256)
}

fn file_data(rel_path: &str, ext: &str, mtime: i64, sha256: String, content: &str) -> FileData {
    let language = CodeIntelEngine::global()
        .infer_language_from_ext(ext)
        .unwrap_or("unknown");
    FileData {
        path: rel_path.to_string(),
        sha256,
        mtime,
        language: language.to_string(),
        symbols: extract_symbols(content, rel_path, ext),
        deps: extract_deps(content, ext),
        edges: extract_edges(content, rel_path, ext),
    }
}

fn scan_changed_file(
    project_root: &Path,
    rel_path: &str,
    ext: &str,
    existing: &HashMap<String, (String, i64)>,
    force: bool,
) -> Option<FileData> {
    let abs_path = project_root.join(rel_path);
    let mtime = file_mtime(&abs_path, rel_path)?;
    if !force && unchanged_by_mtime(existing, rel_path, mtime) {
        return None;
    }
    let content = read_source(&abs_path, rel_path)?;
    let sha256 = hex::encode(Sha256::digest(content.as_bytes()));
    if !force && unchanged_by_hash(existing, rel_path, &sha256) {
        return None;
    }
    Some(file_data(rel_path, ext, mtime, sha256, &content))
}

fn scan_project(
    project_root: &Path,
    existing: &HashMap<String, (String, i64)>,
    force: bool,
) -> ProjectScan {
    let (all_paths, infos) = list_supported_files(project_root);
    let changed = infos
        .par_iter()
        .filter_map(|(rel_path, ext)| {
            scan_changed_file(project_root, rel_path, ext, existing, force)
        })
        .collect();
    ProjectScan { all_paths, changed }
}

fn has_base_changes(existing: &HashMap<String, (String, i64)>, scan: &ProjectScan) -> bool {
    if !scan.changed.is_empty() {
        return true;
    }
    let known: HashSet<&str> = scan.all_paths.iter().map(String::as_str).collect();
    existing.keys().any(|path| !known.contains(path.as_str()))
}

fn apply_scan(
    db: &CodeIndexDb,
    project_root: &Path,
    existing: &HashMap<String, (String, i64)>,
    scan: &ProjectScan,
) -> Result<(), String> {
    if has_base_changes(existing, scan) {
        db.apply_base_write(&scan.changed, &scan.all_paths)?;
        materialize_resolved_edges(db, Some(project_root))?;
        return Ok(());
    }
    if db.has_resolved_edges()? {
        return Ok(());
    }
    materialize_resolved_edges(db, Some(project_root))?;
    Ok(())
}

fn code_index_stats(db: &CodeIndexDb, scan: &ProjectScan) -> Result<CodeIndexStats, String> {
    Ok(CodeIndexStats {
        files_scanned: scan.all_paths.len(),
        files_changed: scan.changed.len(),
        symbols_indexed: scan.changed.iter().map(|f| f.symbols.len()).sum(),
        deps_indexed: scan.changed.iter().map(|f| f.deps.len()).sum(),
        edges_indexed: scan.changed.iter().map(|f| f.edges.len()).sum(),
        total_symbols: db.count_symbols()?,
        total_deps: db.count_deps()?,
        total_edges: db.count_edges()?,
        errors: Vec::new(),
    })
}

pub fn materialize_resolved_edges(
    db: &CodeIndexDb,
    project_root: Option<&Path>,
) -> Result<usize, String> {
    let mut snapshot = CodeIndexSnapshot::from_db(db)?;
    if let Some(root) = project_root {
        snapshot.ts_context =
            Some(crate::services::ts_config_resolver::TsResolutionContext::discover(root));
    }
    let resolved = resolve_code_edges(&snapshot);
    let count = resolved.len();
    db.replace_resolved_edges(&resolved)?;
    Ok(count)
}

pub fn scan_file_metadata(project_root: &Path) -> Result<(usize, i64), String> {
    let engine = CodeIntelEngine::global();
    let mut scan = ProjectMetadata::default();
    for entry in code_walk(project_root).filter_map(Result::ok) {
        if !entry.file_type().is_some_and(|t| t.is_file()) {
            continue;
        }
        let Some(_ext) = source_extension(engine, entry.path()) else {
            continue;
        };
        scan.count = scan.count.wrapping_add(1);
        scan.max_mtime = scan.max_mtime.max(entry_mtime(entry.path()));
    }
    Ok((scan.count, scan.max_mtime))
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

        let stats = rebuild_code_index(&db_path, dir.path(), false).unwrap();
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

        let db_path = dir.path().join("code.db");
        let stats = rebuild_code_index(&db_path, dir.path(), false).unwrap();
        assert_eq!(stats.files_scanned, 1, "should find 1 file");
        assert_eq!(stats.symbols_indexed, 3, "hello, User, Status");
        assert_eq!(stats.deps_indexed, 0);
        assert_eq!(stats.total_symbols, 3);
        assert_eq!(stats.total_deps, 0);

        let db = CodeIndexDb::open(db_path).unwrap();
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

        let db_path = dir.path().join("code.db");
        let first = rebuild_code_index(&db_path, dir.path(), false).unwrap();
        assert_eq!(first.files_scanned, 1);
        assert_eq!(first.symbols_indexed, 1, "first build indexes 1 symbol");

        let second = rebuild_code_index(&db_path, dir.path(), false).unwrap();
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

        let db_path = dir.path().join("code.db");
        rebuild_code_index(&db_path, dir.path(), false).unwrap();

        fs::write(src.join("b.rs"), "pub fn b() {}").unwrap();
        let stats = rebuild_code_index(&db_path, dir.path(), false).unwrap();

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

        let db_path = dir.path().join("code.db");
        rebuild_code_index(&db_path, dir.path(), false).unwrap();

        fs::write(&file_path, "pub fn original() {}\npub fn added() {}").unwrap();
        let stats = rebuild_code_index(&db_path, dir.path(), false).unwrap();

        assert_eq!(stats.files_scanned, 1, "still 1 file");
        assert_eq!(
            stats.symbols_indexed, 2,
            "both original and added symbols re-indexed"
        );
        assert_eq!(stats.total_symbols, 2, "totals reflect re-indexed file");

        let db = CodeIndexDb::open(db_path).unwrap();
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

        let db_path = dir.path().join("code.db");
        let first = rebuild_code_index(&db_path, dir.path(), false).unwrap();
        assert_eq!(first.files_scanned, 2);
        assert_eq!(first.symbols_indexed, 2);

        fs::remove_file(src.join("gone.rs")).unwrap();
        let second = rebuild_code_index(&db_path, dir.path(), false).unwrap();

        assert_eq!(second.files_scanned, 1, "only keep.rs remains");
        assert_eq!(
            second.symbols_indexed, 0,
            "hash-skip: keep.rs unchanged; gone.rs stale-deleted"
        );
        assert_eq!(second.total_symbols, 1, "stale symbols purged");

        let db = CodeIndexDb::open(db_path).unwrap();
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

        let db_path = dir.path().join("code.db");
        let first = rebuild_code_index(&db_path, dir.path(), false).unwrap();
        assert_eq!(first.total_symbols, 1);
        assert_eq!(first.symbols_indexed, 1);

        let forced = rebuild_code_index(&db_path, dir.path(), true).unwrap();
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

        let db_path = dir.path().join("code.db");
        rebuild_code_index(&db_path, dir.path(), false).unwrap();

        let stats = rebuild_code_index(&db_path, dir.path(), false).unwrap();
        assert_eq!(stats.symbols_indexed, 0, "delta is 0 on no-change run");
        assert_eq!(stats.deps_indexed, 0, "delta deps is 0 on no-change run");
        assert_eq!(stats.total_symbols, 2, "totals reflect full DB state");
        assert_eq!(stats.total_deps, 2, "totals reflect full DB state");
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn no_op_rebuild_preserves_resolved_edges() {
        use crate::services::graph_resolver::ResolvedCodeEdge;
        use wm_engine::models::edge_type_model::EdgeProvenance;

        let dir = tempdir().unwrap();
        create_test_file(dir.path(), "src/lib.rs", "pub fn f() {}");
        let db_path = dir.path().join("code.db");
        rebuild_code_index(&db_path, dir.path(), false).unwrap();

        let db = CodeIndexDb::open(db_path.clone()).unwrap();
        let sentinel = ResolvedCodeEdge {
            edge_type: "calls".into(),
            source_file: "sentinel.rs".into(),
            source_symbol: None,
            target_file: "sentinel_target.rs".into(),
            target_symbol: None,
            line: 1,
            provenance: EdgeProvenance::Explicit,
            via: Vec::new(),
        };
        db.replace_resolved_edges(&[sentinel]).unwrap();
        drop(db);

        let stats = rebuild_code_index(&db_path, dir.path(), false).unwrap();
        assert_eq!(stats.files_changed, 0);
        assert_eq!(stats.symbols_indexed, 0);

        let db = CodeIndexDb::open(db_path).unwrap();
        let edges = db.load_resolved_edges().unwrap();
        assert_eq!(edges.len(), 1);
        assert_eq!(
            edges[0].source_file, "sentinel.rs",
            "no-op rebuild must not rewrite resolved edges"
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn no_op_rebuild_repairs_missing_resolved_edges() {
        let dir = tempdir().unwrap();
        create_test_file(dir.path(), "src/lib.rs", "pub fn helper() {}\n");
        create_test_file(dir.path(), "src/main.rs", "pub fn caller() { helper(); }\n");
        let db_path = dir.path().join("code.db");
        rebuild_code_index(&db_path, dir.path(), false).unwrap();

        let db = CodeIndexDb::open(db_path.clone()).unwrap();
        assert!(db.has_resolved_edges().unwrap());
        db.replace_resolved_edges(&[]).unwrap();
        drop(db);

        rebuild_code_index(&db_path, dir.path(), false).unwrap();
        let db = CodeIndexDb::open(db_path).unwrap();
        assert!(
            db.has_resolved_edges().unwrap(),
            "a crash that emptied resolved edges must heal on the next rebuild"
        );
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
        create_test_file(dir.path(), ".gitignore", "node_modules/\ntarget/\n");
        for skip_dir in &[".git", "node_modules", "target"] {
            let p = dir.path().join(skip_dir).join("lib.rs");
            fs::create_dir_all(p.parent().unwrap()).unwrap();
            fs::write(&p, "pub fn f() {}").unwrap();
        }

        let db_path = dir.path().join("code.db");
        let stats = rebuild_code_index(&db_path, dir.path(), false).unwrap();
        assert_eq!(
            stats.files_scanned, 0,
            "all files are in skipped directories"
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn gitignored_source_never_reaches_the_index() {
        let dir = tempdir().unwrap();
        create_test_file(dir.path(), ".gitignore", "ignored/\n");
        create_test_file(
            dir.path(),
            "ignored/probe.rs",
            "pub fn ignored_fn() {}\npub fn caller_link() {}\n",
        );
        create_test_file(
            dir.path(),
            "src/lib.rs",
            "pub fn caller() { ignored_fn(); caller_link(); }\n",
        );
        create_test_file(dir.path(), "src/other.rs", "pub fn other() { caller(); }\n");

        let db_path = dir.path().join("code.db");
        rebuild_code_index(&db_path, dir.path(), false).unwrap();

        let db = CodeIndexDb::open(db_path).unwrap();
        let files = db.list_files().unwrap();
        let mut sorted = files.clone();
        sorted.sort();
        assert_eq!(
            sorted,
            vec!["src/lib.rs".to_string(), "src/other.rs".to_string()]
        );

        let symbols = db
            .query_symbols(None, None, None, None, None, None)
            .unwrap();
        assert!(
            symbols.iter().all(|s| !s.file.starts_with("ignored/")),
            "gitignored symbols must be absent"
        );

        let edges = db.load_resolved_edges().unwrap();
        assert!(
            edges.iter().any(|e| e.source_file == "src/other.rs"),
            "resolution must produce a real edge: {:?}",
            edges
        );
        assert!(
            edges.iter().all(|e| !e.source_file.starts_with("ignored/")
                && !e.target_file.starts_with("ignored/")),
            "gitignored files must appear in no resolved edge: {:?}",
            edges
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn walk_includes_tracked_hidden_and_excludes_reserved() {
        use crate::services::ignore_policy_service::code_walk;

        let dir = tempdir().unwrap();
        create_test_file(dir.path(), ".gitignore", "target/\nnode_modules/\n");
        create_test_file(dir.path(), ".github/workflows/build.rs", "pub fn ci() {}\n");
        create_test_file(dir.path(), ".wm/wiki/probe.rs", "pub fn wiki() {}\n");
        create_test_file(dir.path(), "target/gen.rs", "pub fn t() {}\n");
        create_test_file(
            dir.path(),
            "node_modules/pkg/index.ts",
            "export const x = 1;\n",
        );
        create_test_file(dir.path(), "src/lib.rs", "pub fn s() {}\n");

        let names: Vec<String> = code_walk(dir.path())
            .filter_map(Result::ok)
            .filter(|e| e.file_type().is_some_and(|t| t.is_file()))
            .map(|e| {
                e.path()
                    .strip_prefix(dir.path())
                    .unwrap_or(e.path())
                    .to_string_lossy()
                    .to_string()
            })
            .collect();

        assert!(names.contains(&".github/workflows/build.rs".to_string()));
        assert!(names.contains(&"src/lib.rs".to_string()));
        assert!(!names.iter().any(|n| n.starts_with("target/")));
        assert!(!names.iter().any(|n| n.starts_with("node_modules/")));
        assert!(!names.iter().any(|n| n.starts_with(".wm/")));
    }
}
