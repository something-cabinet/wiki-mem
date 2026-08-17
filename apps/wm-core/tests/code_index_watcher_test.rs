//! Phase 1 acceptance tests for spec `wiki:specs:code-edge-resolution`.
//!
//! AC-1.1 — a source edit is reflected in code-edge queries without a manual
//!          `wm index code`.
//! AC-1.2 — a deleted source file loses its symbols and edges without a
//!          manual rebuild.
//! AC-1.5 — the refresh is exercised through the real watcher thread (write to
//!          disk, poll with a deadline), never by calling the handler directly.
//!
//! Skip-list scoping and the read-time staleness probe are covered here too:
//! the probe is what makes AC-1.1 reachable for one-shot CLI invocations, which
//! construct an engine, run, and exit before any watcher event arrives.

#![cfg(feature = "code-intel")]

use std::path::Path;
use std::time::{Duration, Instant};

use wm_code_intel::services::code_index_db::CodeIndexDb;
use wm_code_intel::services::ingest_service::rebuild_code_index;

use wm_core::config::ProjectConfig;
use wm_core::engine::code_index_refresh_service::{refresh_code_index, refresh_if_stale};
use wm_core::engine::MainEngine;

const POLL_DEADLINE: Duration = Duration::from_secs(20);
const POLL_INTERVAL: Duration = Duration::from_millis(200);

fn setup_code_project() -> (tempfile::TempDir, std::path::PathBuf) {
    let dir = tempfile::tempdir().expect("temp dir");
    let root = dir.path().to_path_buf();

    let wm_dir = root.join(".wm");
    std::fs::create_dir_all(wm_dir.join("wiki").join("concepts")).expect("create wiki dir");
    std::fs::create_dir_all(wm_dir.join("state")).expect("create state dir");
    std::fs::create_dir_all(root.join("src")).expect("create src dir");

    let config = serde_json::json!({
        "schema_version": 1,
        "embedding": { "model_name": "none", "dimensions": 384, "batch_size": 32 },
        "permissions": { "preset": "read-write" },
        "custom_edge_types": [],
        "source_dirs": ["docs/", "specs/"],
        "source_extensions": ["md", "yaml", "txt"],
        "search": {
            "default_mode": "keyword",
            "default_limit": 20,
            "rrf_k": 60,
            "scoring": { "debounce_ms": 500 }
        }
    });
    std::fs::write(
        wm_dir.join("config.json"),
        serde_json::to_string_pretty(&config).unwrap(),
    )
    .expect("write config.json");

    std::fs::write(
        root.join("src").join("lib.rs"),
        "pub fn seed_function() -> u32 { 1 }\n",
    )
    .expect("write seed source");

    (dir, root)
}

fn load_config(root: &Path) -> ProjectConfig {
    std::fs::read_to_string(root.join(".wm").join("config.json"))
        .ok()
        .and_then(|s| serde_json::from_str::<ProjectConfig>(&s).ok())
        .unwrap_or_default()
}

fn code_db_path(root: &Path) -> std::path::PathBuf {
    root.join(".wm").join("state").join("code.db")
}

/// Build the index once so the refresh paths have an existing index to update.
/// Code intelligence stays opt-in: nothing auto-creates `code.db`.
fn build_initial_index(root: &Path) {
    let db = CodeIndexDb::open(code_db_path(root)).expect("open code db");
    rebuild_code_index(&db, root, false).expect("initial index build");
}

fn indexed_symbol_names(root: &Path) -> Vec<String> {
    let db = match CodeIndexDb::open(code_db_path(root)) {
        Ok(db) => db,
        Err(_) => return Vec::new(),
    };
    db.query_symbols(None, None, None, None, None, None)
        .map(|syms| syms.into_iter().map(|s| s.name).collect())
        .unwrap_or_default()
}

fn poll_until<F: Fn() -> bool>(what: &str, predicate: F) {
    let deadline = Instant::now() + POLL_DEADLINE;
    while Instant::now() < deadline {
        if predicate() {
            return;
        }
        std::thread::sleep(POLL_INTERVAL);
    }
    panic!("timed out waiting for {}", what);
}

#[tokio::test(flavor = "multi_thread")]
async fn watcher_indexes_new_source_file() {
    let (_dir, root) = setup_code_project();
    build_initial_index(&root);

    let engine = MainEngine::with_root(load_config(&root), root.clone());

    std::fs::write(
        root.join("src").join("added.rs"),
        "pub fn freshly_added_symbol() -> u32 { 7 }\n",
    )
    .expect("write new source file");

    poll_until("watcher to index freshly_added_symbol", || {
        indexed_symbol_names(&root)
            .iter()
            .any(|n| n == "freshly_added_symbol")
    });

    drop(engine);
}

#[tokio::test(flavor = "multi_thread")]
async fn watcher_removes_deleted_source_file() {
    let (_dir, root) = setup_code_project();
    let doomed = root.join("src").join("doomed.rs");
    std::fs::write(&doomed, "pub fn doomed_symbol() -> u32 { 3 }\n").expect("write doomed source");
    build_initial_index(&root);

    assert!(
        indexed_symbol_names(&root)
            .iter()
            .any(|n| n == "doomed_symbol"),
        "doomed_symbol should be indexed before deletion"
    );

    let engine = MainEngine::with_root(load_config(&root), root.clone());

    std::fs::remove_file(&doomed).expect("delete source file");

    poll_until("watcher to drop doomed_symbol", || {
        !indexed_symbol_names(&root)
            .iter()
            .any(|n| n == "doomed_symbol")
    });

    drop(engine);
}

#[tokio::test(flavor = "multi_thread")]
async fn skipped_dirs_never_reach_the_index() {
    let (_dir, root) = setup_code_project();
    std::fs::create_dir_all(root.join("target")).expect("create target dir");
    build_initial_index(&root);

    let engine = MainEngine::with_root(load_config(&root), root.clone());

    std::fs::write(
        root.join("target").join("generated.rs"),
        "pub fn generated_artifact_symbol() -> u32 { 9 }\n",
    )
    .expect("write generated source");
    std::fs::write(
        root.join("src").join("tracked.rs"),
        "pub fn tracked_symbol() -> u32 { 11 }\n",
    )
    .expect("write tracked source");

    poll_until("watcher to index tracked_symbol", || {
        indexed_symbol_names(&root)
            .iter()
            .any(|n| n == "tracked_symbol")
    });

    let names = indexed_symbol_names(&root);
    assert!(
        !names.iter().any(|n| n == "generated_artifact_symbol"),
        "symbols under target/ must never be indexed, found: {:?}",
        names
    );

    drop(engine);
}

#[test]
fn refresh_if_stale_reindexes_after_external_edit() {
    let (_dir, root) = setup_code_project();
    build_initial_index(&root);

    std::fs::write(
        root.join("src").join("external.rs"),
        "pub fn externally_added_symbol() -> u32 { 5 }\n",
    )
    .expect("write external source");

    let rebuilt = refresh_if_stale(&root).expect("staleness probe");
    assert!(rebuilt, "probe should detect the new file and rebuild");

    assert!(
        indexed_symbol_names(&root)
            .iter()
            .any(|n| n == "externally_added_symbol"),
        "externally added symbol should be indexed after the staleness probe"
    );
}

#[test]
fn code_graph_read_refreshes_a_stale_index() {
    let (_dir, root) = setup_code_project();
    build_initial_index(&root);

    std::fs::write(
        root.join("src").join("late.rs"),
        "pub fn late_arriving_symbol() -> u32 { 13 }\n",
    )
    .expect("write late source");

    let graph = wm_core::graph::code_edges::load_code_graph(&root).expect("load code graph");
    assert!(
        graph.is_some(),
        "an existing index should yield a code graph"
    );

    assert!(
        indexed_symbol_names(&root)
            .iter()
            .any(|n| n == "late_arriving_symbol"),
        "reading the code graph must refresh a stale index — otherwise a one-shot \
         CLI invocation answers from whatever `wm index code` last wrote"
    );
}

#[test]
fn refresh_if_stale_is_noop_when_index_is_current() {
    let (_dir, root) = setup_code_project();
    build_initial_index(&root);

    let rebuilt = refresh_if_stale(&root).expect("staleness probe");
    assert!(!rebuilt, "a current index must not be rebuilt");
}

#[test]
fn refresh_leaves_absent_index_absent() {
    let (_dir, root) = setup_code_project();

    let stats = refresh_code_index(&root).expect("refresh without index");
    assert!(
        stats.is_none(),
        "code intelligence is opt-in — refresh must not create an index that was never built"
    );
    assert!(
        !code_db_path(&root).exists(),
        "no code.db should be created by a refresh"
    );
}
