#![cfg(feature = "code-intel")]

use std::fs;
use std::path::Path;

use wm_code_intel::services::code_index_db::CodeIndexDb;
use wm_code_intel::services::ingest_service::{rebuild_code_index, scan_file_metadata};

#[path = "helpers/cli_run.rs"]
mod helpers;

#[path = "helpers/setup.rs"]
mod setup;
use setup::setup_test_project;

#[path = "helpers/macros.rs"]
mod _macros;

fn create_source(dir: &Path, rel_path: &str, content: &str) {
    let full = dir.join(rel_path);
    fs::create_dir_all(full.parent().unwrap()).unwrap();
    fs::write(&full, content).unwrap();
}

#[test]
fn e2e_code_empty_project() {
    let (_dir, root) = setup_test_project();
    let db_path = root.join(".wm").join("state").join("code.db");
    fs::create_dir_all(db_path.parent().unwrap()).unwrap();
    let db = CodeIndexDb::open(db_path).unwrap();

    let stats = rebuild_code_index(&db, &root, false).unwrap();
    assert_eq!(stats.files_scanned, 0);
    assert_eq!(stats.symbols_indexed, 0);
    assert_eq!(stats.deps_indexed, 0);

    let (count, _) = db.get_file_count_and_max_mtime().unwrap();
    assert_eq!(count, 0);
}

#[test]
fn e2e_code_rebuild_and_query_symbols() {
    let (_dir, root) = setup_test_project();

    create_source(
        &root,
        "src/lib.rs",
        r#"
pub fn greet(name: &str) -> String { format!("Hello, {}!", name) }
pub struct User { name: String, }
pub enum Status { Active, Inactive }
"#,
    );
    create_source(&root, "src/utils.rs", r#"pub fn helper() -> u32 { 42 }"#);

    let db_path = root.join(".wm").join("state").join("code.db");
    fs::create_dir_all(db_path.parent().unwrap()).unwrap();
    let db = CodeIndexDb::open(db_path).unwrap();
    let stats = rebuild_code_index(&db, &root, false).unwrap();

    assert_eq!(stats.files_scanned, 2, "should index 2 files");
    assert_eq!(
        stats.symbols_indexed, 4,
        "should find 4 symbols: greet, User, Status, helper"
    );
    assert_eq!(stats.deps_indexed, 0);

    assert_eq!(
        db.query_symbols(None, None, None, None, None, None)
            .unwrap()
            .len(),
        4
    );

    let user_syms = db
        .query_symbols(Some("User"), None, None, None, None, None)
        .unwrap();
    assert_eq!(user_syms.len(), 1);
    assert_eq!(user_syms[0].name, "User");

    assert_eq!(
        db.query_symbols(None, Some("function"), None, None, None, None)
            .unwrap()
            .len(),
        2,
        "greet + helper"
    );

    assert_eq!(
        db.query_symbols(None, None, None, None, Some("rust"), None)
            .unwrap()
            .len(),
        4
    );
}

#[test]
fn e2e_code_deps() {
    let (_dir, root) = setup_test_project();

    create_source(
        &root,
        "src/engine.rs",
        r#"
use std::collections::HashMap;
use crate::models::User;
use tokio::runtime;
"#,
    );
    create_source(
        &root,
        "src/models.rs",
        r#"
use serde::{Serialize, Deserialize};
"#,
    );

    let db_path = root.join(".wm").join("state").join("code.db");
    fs::create_dir_all(db_path.parent().unwrap()).unwrap();
    let db = CodeIndexDb::open(db_path).unwrap();
    let stats = rebuild_code_index(&db, &root, false).unwrap();

    assert_eq!(stats.files_scanned, 2);
    assert!(stats.deps_indexed > 0, "should find import declarations");

    let engine_deps = db
        .query_deps(Some("src/engine.rs"), None, false, Some(1), None)
        .unwrap();
    assert!(!engine_deps.is_empty(), "engine.rs should have deps");

    let rev = db
        .query_deps(None, Some("HashMap"), true, Some(1), None)
        .unwrap();
    assert!(!rev.is_empty(), "reverse deps should find engine.rs");
    let has_engine = rev.iter().any(|v| {
        v.get("file")
            .and_then(|f| f.as_str())
            .map_or(false, |f| f.contains("engine"))
    });
    assert!(has_engine, "reverse deps should include engine.rs");
}

#[test]
fn e2e_code_incremental() {
    let (_dir, root) = setup_test_project();

    create_source(&root, "src/lib.rs", "pub fn first() -> u32 { 1 }");
    create_source(&root, "src/other.rs", "pub fn second() -> u32 { 2 }");
    let db_path = root.join(".wm").join("state").join("code.db");
    fs::create_dir_all(db_path.parent().unwrap()).unwrap();
    let db = CodeIndexDb::open(db_path.clone()).unwrap();
    let first = rebuild_code_index(&db, &root, false).unwrap();
    assert_eq!(first.files_scanned, 2);
    assert_eq!(first.symbols_indexed, 2);

    let second = rebuild_code_index(&db, &root, false).unwrap();
    assert_eq!(second.files_scanned, 2);
    assert_eq!(second.symbols_indexed, 0, "0 new symbols — hash-skip");

    create_source(
        &root,
        "src/lib.rs",
        "pub fn first() -> u32 { 1 }\npub fn third() -> u32 { 3 }",
    );
    let third = rebuild_code_index(&db, &root, false).unwrap();
    assert_eq!(third.files_scanned, 2);
    assert_eq!(
        third.symbols_indexed, 2,
        "file changed — both symbols re-indexed (first+third=2)"
    );
    assert_eq!(
        db.query_symbols(None, None, None, None, None, None)
            .unwrap()
            .len(),
        3
    );

    fs::remove_file(root.join("src/other.rs")).unwrap();
    let fourth = rebuild_code_index(&db, &root, false).unwrap();
    assert_eq!(fourth.files_scanned, 1);
    assert_eq!(fourth.symbols_indexed, 0);
    assert_eq!(
        db.query_symbols(None, None, None, None, None, None)
            .unwrap()
            .len(),
        2
    );
}

#[test]
fn e2e_code_stale_detection() {
    let (_dir, root) = setup_test_project();

    create_source(&root, "src/lib.rs", "pub fn stable() -> u32 { 42 }");
    let db_path = root.join(".wm").join("state").join("code.db");
    fs::create_dir_all(db_path.parent().unwrap()).unwrap();
    let db = CodeIndexDb::open(db_path).unwrap();
    rebuild_code_index(&db, &root, false).unwrap();

    let (cached_count, cached_mtime) = db.get_file_count_and_max_mtime().unwrap();
    let (actual_count, actual_mtime) = scan_file_metadata(&root).unwrap();
    assert!(
        !(cached_count != actual_count || cached_mtime < actual_mtime),
        "not stale before edit"
    );

    create_source(&root, "src/new.rs", "pub fn new_func() -> u32 { 7 }");
    let (actual_count2, actual_mtime2) = scan_file_metadata(&root).unwrap();
    assert!(
        cached_count != actual_count2 || cached_mtime < actual_mtime2,
        "stale after add"
    );
}

#[test]
fn e2e_code_multi_language() {
    let (_dir, root) = setup_test_project();

    create_source(&root, "src/lib.rs", "pub fn rust_func() {}");
    create_source(&root, "src/main.ts", "export function tsFunc(): void {}");
    create_source(&root, "src/main.py", "def py_func():\n    pass");
    create_source(&root, "src/main.go", "package main\nfunc goFunc() {}");

    let db_path = root.join(".wm").join("state").join("code.db");
    fs::create_dir_all(db_path.parent().unwrap()).unwrap();
    let db = CodeIndexDb::open(db_path).unwrap();
    let stats = rebuild_code_index(&db, &root, false).unwrap();
    assert_eq!(stats.files_scanned, 4);
    assert_eq!(stats.symbols_indexed, 4);

    assert_eq!(
        db.query_symbols(None, None, None, None, Some("rust"), None)
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        db.query_symbols(None, None, None, None, Some("typescript"), None)
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        db.query_symbols(None, None, None, None, Some("python"), None)
            .unwrap()
            .len(),
        1
    );
    let go = db
        .query_symbols(None, None, None, None, Some("go"), None)
        .unwrap();
    assert!(go.len() >= 1);
}

#[test]
fn e2e_code_unsupported_extensions_skipped() {
    let (_dir, root) = setup_test_project();
    create_source(&root, "src/lib.rs", "pub fn ok() {}");
    create_source(&root, "src/styles.css", ".cls { color: red; }");
    create_source(&root, "README.md", "# Project");

    let db_path = root.join(".wm").join("state").join("code.db");
    fs::create_dir_all(db_path.parent().unwrap()).unwrap();
    let db = CodeIndexDb::open(db_path).unwrap();
    let stats = rebuild_code_index(&db, &root, false).unwrap();
    assert_eq!(stats.files_scanned, 1, "only .rs should be indexed");
    assert_eq!(stats.symbols_indexed, 1);
}

#[test]
fn e2e_code_cli_index_code() {
    let (_dir, root) = setup_test_project();
    create_source(&root, "src/lib.rs", "pub fn cli_func() -> u32 { 99 }");

    let res = helpers::run_cli(&root, &["index", "code"]);
    assert_success!(res);

    let db_path = root.join(".wm").join("state").join("code.db");
    assert!(db_path.exists(), "code.db should exist after wm index code");

    let db = CodeIndexDb::open(db_path).unwrap();
    let results = db
        .query_symbols(None, None, None, None, None, None)
        .unwrap();
    let names: Vec<&str> = results.iter().map(|s| s.name.as_str()).collect();
    assert!(names.contains(&"cli_func"), "should find cli_func symbol");
}
