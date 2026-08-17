//! Stress and scale tests (wiki task `stress`, oracle D1 re-spec).
//!
//! The daemon owns the engine + read-only web API + Angular SPA; tool dispatch
//! has no HTTP surface (wm-server is web-UI-only). Concurrent load therefore
//! means concurrent filesystem writes + concurrent web-API reads against the
//! live daemon, whose file watcher indexes the writes. Heavy benchmarks stay
//! `#[ignore]`d and are run with the `stress` release-test runner:
//!
//! ```bash
//! # default suite (fast; concurrent daemon test runs in CI):
//! cargo test -p wm-core --test stress_test
//! # heavy benchmarks (10K-doc search, 1000-page graph rebuild, compaction):
//! cargo test -p wm-core --test stress_test -- --ignored
//! ```

#[path = "helpers/cli.rs"]
mod helpers;
use helpers::{run_cli, run_cli_with_stdin};

#[path = "helpers/setup.rs"]
mod setup;
use setup::setup_test_project;

#[path = "helpers/macros.rs"]
mod _macros;

#[path = "helpers/http_daemon.rs"]
mod daemon;
use daemon::DaemonHandle;

use serde_json::{json, Value};
use std::time::{Duration, Instant};

#[test]
fn test_concurrent_daemon_connections() {
    let (_dir, root) = setup_test_project();
    let daemon = std::sync::Arc::new(DaemonHandle::start(&root));

    const N: usize = 10;
    let mut handles = Vec::with_capacity(N);
    for i in 0..N {
        let daemon = std::sync::Arc::clone(&daemon);
        let root = root.clone();
        handles.push(std::thread::spawn(move || {
            if i % 2 == 0 {
                let path = root
                    .join(".wm")
                    .join("wiki")
                    .join("concepts")
                    .join(format!("conc-{i:02}.md"));
                let body = format!(
                    "---\ntitle: Concurrent {}\ntype: concept\n---\n\nConcurrent connection {} payload.\n",
                    i, i
                );
                let result = std::fs::write(&path, body)
                    .map(|()| json!({ "id": format!("wiki:concepts:conc-{i:02}") }))
                    .map_err(|e| ("IO_ERROR".to_string(), e.to_string()));
                (i, result)
            } else {
                let (status, body) = daemon.web_post("/api/pages/list", &json!({}));
                let result = if status == 200 {
                    Ok(body)
                } else {
                    Err(("HTTP".to_string(), format!("status {status}")))
                };
                (i, result)
            }
        }));
    }

    for handle in handles {
        let (i, result) = handle.join().expect("worker thread panicked");
        match result {
            Ok(data) => {
                if i % 2 == 0 {
                    assert!(
                        data["id"].as_str().is_some(),
                        "writer {i} should report a page id, got {data}"
                    );
                }
            }
            Err((code, msg)) => {
                panic!("concurrent connection {i} failed [{code}]: {msg}");
            }
        }
    }

    let deadline = Instant::now() + Duration::from_secs(15);
    let found = loop {
        let (status, body) = daemon.web_post("/api/pages/list", &json!({}));
        assert_eq!(status, 200, "pages/list should succeed: {body}");
        let pages = body["pages"].as_array().expect("pages array");
        let count = (0..N)
            .filter(|i| i % 2 == 0)
            .filter(|i| {
                let id = format!("wiki:concepts:conc-{i:02}");
                pages.iter().any(|p| p["id"].as_str() == Some(id.as_str()))
            })
            .count();
        if count == N / 2 {
            break count;
        }
        assert!(
            Instant::now() < deadline,
            "data integrity: expected {} concurrent pages to be indexed, found {count} in {pages:?}",
            N / 2
        );
        std::thread::sleep(Duration::from_millis(250));
    };
    assert_eq!(found, N / 2, "all concurrent writes must survive");

    let (status, body) = daemon.raw("GET", "/api/health", &json!({}), None);
    assert_eq!(status, 200, "daemon should still be alive and serving after the burst: {body}");
}

#[test]
#[ignore]
fn test_10k_doc_search_benchmark() {
    let (_dir, root) = setup_test_project();

    const COUNT: usize = 10_000;
    let concepts_dir = root.join(".wm").join("wiki").join("concepts");
    std::fs::create_dir_all(&concepts_dir).expect("create concepts dir");
    for i in 0..COUNT {
        let body = format!(
            "---\ntitle: Bench {}\ntype: concept\nstatus: draft\nid: wiki:concepts:bench-{:05}\ntags: [bench]\n---\n\n\
             Benchmark document {}: the quasar-{}-token appears here alongside shared corpus vocabulary for the 10K search benchmark.\n",
            i, i, i, i
        );
        std::fs::write(
            concepts_dir.join(format!("bench-{i:05}.md")),
            body,
        )
        .expect("write benchmark doc");
    }

    let daemon = DaemonHandle::start(&root);

    let resp = daemon.web_post(
        "/api/search/query",
        &json!({ "q": "quasar-9999", "type": "all", "limit": 10 }),
    );
    let body = parse_search(&resp);
    assert!(
        !body["results"].as_array().is_none_or(|r| r.is_empty()),
        "warm search should return results, got {body}"
    );

    let start = Instant::now();
    let resp = daemon.web_post(
        "/api/search/query",
        &json!({ "q": "quasar-4242", "type": "all", "limit": 10 }),
    );
    let elapsed = start.elapsed();
    let body = parse_search(&resp);
    assert!(
        !body["results"].as_array().is_none_or(|r| r.is_empty()),
        "search should return results, got {body}"
    );
    assert!(
        elapsed < std::time::Duration::from_millis(500),
        "10K-doc search took {elapsed:?} (expected <500ms)"
    );
}

fn parse_search(resp: &(u16, Value)) -> Value {
    let (status, body) = resp;
    assert!(
        (200..300).contains(status),
        "search returned HTTP {status}: {body}"
    );
    assert_eq!(
        body.get("success").and_then(Value::as_bool),
        Some(true),
        "search should succeed, got {body}"
    );
    body.clone()
}

#[test]
#[ignore]
fn test_1000_page_graph_rebuild() {
    let (_dir, root) = setup_test_project();

    for i in 0..1000 {
        let res = run_cli_with_stdin(
            &root,
            &[
                "page",
                "create",
                &format!("concepts/page-{}", i),
                &format!("Page {}", i),
            ],
            &format!(
                "Content for page {} with some searchable text for benchmark purposes.",
                i
            ),
        );
        assert_success!(res);
    }

    let start = Instant::now();
    let res = run_cli(&root, &["index", "rebuild"]);
    let duration = start.elapsed();
    assert_success!(res);
    assert!(
        duration.as_secs() < 5,
        "graph rebuild took {:.1}s (expected <5s)",
        duration.as_secs_f64()
    );

    let res = run_cli(&root, &["graph", "stats", "--json"]);
    assert_success!(res);
}

#[test]
#[ignore]
fn test_version_compaction() {
    let (_dir, root) = setup_test_project();

    let res = run_cli_with_stdin(
        &root,
        &["page", "create", "tasks/compact-test", "Original Title"],
        "Version compaction test.",
    );
    assert_success!(res);

    let page_path = root
        .join(".wm")
        .join("wiki")
        .join("tasks")
        .join("compact-test.md");
    for i in 0i32..500 {
        let content = std::fs::read_to_string(&page_path).unwrap_or_default();
        let updated = content.replace(
            &format!("updated {}", i.saturating_sub(1)),
            &format!("updated {}", i),
        );
        let new_content = if i == 0 {
            content.replace("Original Title", &format!("Updated {}", i))
        } else {
            updated
        };
        std::fs::write(&page_path, new_content).expect("write");
    }

    let res = run_cli(&root, &["index", "rebuild"]);
    assert_success!(res);

    let versions_dir = root.join(".wm").join("versions");
    if versions_dir.exists() {
        let mut total_size = 0u64;
        if let Ok(entries) = std::fs::read_dir(&versions_dir) {
            for entry in entries.flatten() {
                if let Ok(meta) = entry.metadata() {
                    total_size += meta.len();
                }
            }
        }
        assert!(
            total_size < 100_000,
            "version files total {total_size} bytes (expected <100KB)"
        );
    }
}
