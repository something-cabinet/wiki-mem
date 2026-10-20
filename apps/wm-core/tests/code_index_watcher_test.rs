#![cfg(feature = "code-intel")]

use std::collections::BTreeSet;
use std::path::Path;
use std::time::{Duration, Instant};

use wm_code_intel::services::code_index_db::CodeIndexDb;
use wm_code_intel::services::ingest_service::rebuild_code_index;

use wm_constants::CODE_INDEX_STALE_FILE;
use wm_core::config::ProjectConfig;
use wm_core::engine::code_index_refresh_service::{
    pending_refresh_failure, refresh_code_index, refresh_code_index_with_retry, refresh_if_stale,
};
use wm_core::engine::MainEngine;

static WATCHER_GUARD: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

const POLL_INTERVAL: Duration = Duration::from_millis(200);
const CONFIRM_READ_INTERVAL: Duration = Duration::from_secs(5);
const BUDGETED_STEPS: u32 = 25;
const BUDGET_FLOOR: Duration = Duration::from_secs(20);
const BUDGET_CEILING: Duration = Duration::from_secs(45);

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

    std::fs::write(root.join(".gitignore"), "target/\nnode_modules/\n").expect("write .gitignore");

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

fn build_initial_index(root: &Path) {
    let db = CodeIndexDb::open(code_db_path(root)).expect("open code db");
    rebuild_code_index(&db, root, false).expect("initial index build");
}

fn indexed_symbol_names(root: &Path) -> Option<Vec<String>> {
    let db = CodeIndexDb::open(code_db_path(root)).ok()?;
    db.query_symbols(None, None, None, None, None, None)
        .ok()
        .map(|syms| syms.into_iter().map(|s| s.name).collect())
}

#[derive(Clone)]
struct IndexSnapshot {
    symbols: BTreeSet<String>,
}

impl IndexSnapshot {
    fn read(root: &Path) -> Option<Self> {
        Some(Self {
            symbols: indexed_symbol_names(root)?.into_iter().collect(),
        })
    }

    fn contains(&self, name: &str) -> bool {
        self.symbols.contains(name)
    }

    fn changed_from(&self, baseline: &Self) -> bool {
        self.symbols != baseline.symbols
    }

    fn names(&self) -> Vec<String> {
        self.symbols.iter().cloned().collect()
    }
}

fn poll_budget(slowest_step: Duration) -> Duration {
    slowest_step
        .saturating_mul(BUDGETED_STEPS)
        .max(BUDGET_FLOOR)
        .min(BUDGET_CEILING)
}

fn state_fingerprint(root: &Path) -> Vec<(String, u64, u128)> {
    let state_dir = root.join(".wm").join("state");
    let entries = match std::fs::read_dir(state_dir) {
        Ok(entries) => entries,
        Err(_) => return Vec::new(),
    };
    let mut fingerprint: Vec<(String, u64, u128)> = entries
        .filter_map(Result::ok)
        .filter_map(|entry| {
            let metadata = entry.metadata().ok()?;
            let modified = metadata.modified().ok()?;
            let nanos = modified
                .duration_since(std::time::UNIX_EPOCH)
                .ok()?
                .as_nanos();
            Some((
                entry.file_name().to_string_lossy().into_owned(),
                metadata.len(),
                nanos,
            ))
        })
        .collect();
    fingerprint.sort();
    fingerprint
}

fn poll_until<F>(what: &str, root: &Path, predicate: F)
where
    F: Fn(&IndexSnapshot) -> bool,
{
    let started = Instant::now();
    let mut steps: u32 = 0;
    let mut reads: u32 = 0;
    let mut failed_reads: u32 = 0;
    let mut changed_reads: u32 = 0;
    let mut file_updates: u32 = 0;
    let mut slowest_step = Duration::ZERO;
    let mut baseline: Option<IndexSnapshot> = None;
    let mut latest: Option<IndexSnapshot> = None;
    let mut last_fingerprint = state_fingerprint(root);
    let mut last_read = started;
    let mut read_pending = true;

    loop {
        let step_started = Instant::now();
        steps += 1;

        let fingerprint = state_fingerprint(root);
        let changed_now = fingerprint != last_fingerprint;
        if changed_now {
            file_updates += 1;
            last_fingerprint = fingerprint;
            read_pending = true;
        }

        let read_due =
            (read_pending && !changed_now) || last_read.elapsed() >= CONFIRM_READ_INTERVAL;
        if read_due {
            reads += 1;
            match IndexSnapshot::read(root) {
                Some(snapshot) => {
                    if predicate(&snapshot) {
                        return;
                    }
                    let reference = baseline.get_or_insert_with(|| snapshot.clone());
                    if snapshot.changed_from(reference) {
                        changed_reads += 1;
                    }
                    latest = Some(snapshot);
                    last_fingerprint = state_fingerprint(root);
                    last_read = Instant::now();
                    read_pending = false;
                }
                None => failed_reads += 1,
            }
        }

        let budget = poll_budget(slowest_step);
        if started.elapsed() >= budget {
            let progress = match changed_reads {
                0 => "no progress: the watcher never altered the index".to_string(),
                changed => format!(
                    "progress but predicate never held: the index changed on {changed} of {reads} reads"
                ),
            };
            let baseline_names = baseline
                .as_ref()
                .map(IndexSnapshot::names)
                .unwrap_or_default();
            let latest_names = latest
                .as_ref()
                .map(IndexSnapshot::names)
                .unwrap_or_default();
            panic!(
                "timed out waiting for {what}: {steps} polls over {:?} with budget {:?} and \
                 slowest step {:?}; index reads {reads} ({failed_reads} failed), index files \
                 changed {file_updates} times; {progress}; baseline symbols {baseline_names:?}, \
                 latest symbols {latest_names:?}",
                started.elapsed(),
                budget,
                slowest_step,
            );
        }

        std::thread::sleep(POLL_INTERVAL);
        slowest_step = slowest_step.max(step_started.elapsed());
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn watcher_indexes_new_source_file() {
    let _guard = WATCHER_GUARD.lock().await;
    let (_dir, root) = setup_code_project();
    build_initial_index(&root);

    let engine = MainEngine::with_root(load_config(&root), root.clone());

    std::fs::write(
        root.join("src").join("added.rs"),
        "pub fn freshly_added_symbol() -> u32 { 7 }\n",
    )
    .expect("write new source file");

    poll_until("watcher to index freshly_added_symbol", &root, |index| {
        index.contains("freshly_added_symbol")
    });

    drop(engine);
}

#[tokio::test(flavor = "multi_thread")]
async fn watcher_removes_deleted_source_file() {
    let _guard = WATCHER_GUARD.lock().await;
    let (_dir, root) = setup_code_project();
    let doomed = root.join("src").join("doomed.rs");
    std::fs::write(&doomed, "pub fn doomed_symbol() -> u32 { 3 }\n").expect("write doomed source");
    build_initial_index(&root);

    assert!(
        indexed_symbol_names(&root).is_some_and(|names| names.iter().any(|n| n == "doomed_symbol")),
        "doomed_symbol should be indexed before deletion"
    );

    let engine = MainEngine::with_root(load_config(&root), root.clone());

    std::fs::remove_file(&doomed).expect("delete source file");

    poll_until("watcher to drop doomed_symbol", &root, |index| {
        !index.contains("doomed_symbol")
    });

    drop(engine);
}

#[tokio::test(flavor = "multi_thread")]
async fn skipped_dirs_never_reach_the_index() {
    let _guard = WATCHER_GUARD.lock().await;
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

    poll_until("watcher to index tracked_symbol", &root, |index| {
        index.contains("tracked_symbol")
    });

    let names = indexed_symbol_names(&root).expect("read index after watcher update");
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
            .is_some_and(|names| names.iter().any(|n| n == "externally_added_symbol")),
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
            .is_some_and(|names| names.iter().any(|n| n == "late_arriving_symbol")),
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

#[test]
fn failed_refresh_is_recorded_then_cleared() {
    let (_dir, root) = setup_code_project();
    let db_path = code_db_path(&root);
    std::fs::create_dir_all(&db_path).expect("block code.db with a directory");

    refresh_code_index_with_retry(&root).expect_err("a blocked database must fail the refresh");

    let failure = pending_refresh_failure(&root).expect("read failure marker");
    assert!(
        failure.is_some(),
        "a dropped refresh must be durably recorded, not only logged"
    );

    std::fs::remove_dir_all(&db_path).expect("remove blocking directory");
    build_initial_index(&root);
    refresh_code_index_with_retry(&root).expect("refresh after repair");

    assert!(
        pending_refresh_failure(&root)
            .expect("read marker after recovery")
            .is_none(),
        "a successful refresh must clear the stale marker"
    );
}

#[test]
fn stale_marker_forces_rebuild_when_metadata_is_current() {
    let (_dir, root) = setup_code_project();
    build_initial_index(&root);
    assert!(
        !refresh_if_stale(&root).expect("initial staleness probe"),
        "a freshly built index must not rebuild"
    );

    let marker = root.join(".wm").join("state").join(CODE_INDEX_STALE_FILE);
    std::fs::write(&marker, "simulated failed refresh").expect("write stale marker");

    assert!(
        refresh_if_stale(&root).expect("staleness probe with marker"),
        "a stale marker must force a rebuild even when file metadata looks current"
    );
    assert!(
        !marker.exists(),
        "a successful rebuild must clear the stale marker"
    );
}
