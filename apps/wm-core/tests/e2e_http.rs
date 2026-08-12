//! HTTP-API E2E suite against the live `wm-server` daemon (oracle D1).
//!
//! The daemon owns the engine, the read-only web API, and the Angular SPA.
//! Tool dispatch no longer has an HTTP surface — wm-server is web-UI-only —
//! so this suite seeds content by writing markdown files directly into the
//! fixture's `.wm/wiki/` and lets the daemon's file watcher index them:
//!
//! - web API surface: health, initial, pages, search, graph, tasks, auth
//! - watcher-backed refresh: disk writes become visible through the web API
//!   without manual index rebuilds
//! - SPA fallback behavior when no bundled frontend exists
//! - SPA cache-control headers when a bundled frontend is present

#[path = "helpers/setup.rs"]
mod setup;
use setup::setup_test_project;

#[path = "helpers/http_daemon.rs"]
mod daemon;
use daemon::DaemonHandle;

use serde_json::{json, Value};
use std::time::{Duration, Instant};

fn parse_ok(resp: &(u16, Value)) -> &Value {
    let (status, body) = resp;
    assert!(
        (200..300).contains(status),
        "expected 2xx, got HTTP {status}: {body}"
    );
    assert_eq!(
        body.get("success").and_then(Value::as_bool),
        Some(true),
        "expected success:true, got {body}"
    );
    body
}

/// Poll the web API until `predicate` holds (bounded). The daemon refreshes
/// its in-memory graph through the file watcher after files are written, so
/// assertions that depend on seeded content must tolerate the debounce.
fn wait_until(daemon: &DaemonHandle, mut predicate: impl FnMut(&DaemonHandle) -> bool) {
    let deadline = Instant::now() + Duration::from_secs(15);
    while !predicate(daemon) {
        assert!(
            Instant::now() < deadline,
            "daemon did not index the seeded files within 15s"
        );
        std::thread::sleep(Duration::from_millis(250));
    }
}

/// Write a wiki page on disk. The daemon's file watcher picks it up and
/// refreshes the in-memory graph (the belt-and-suspenders to the write path).
fn seed_page(root: &std::path::Path, rel: &str, frontmatter: &str, body: &str) {
    let path = root.join(".wm").join("wiki").join(rel);
    std::fs::create_dir_all(path.parent().expect("wiki page has a parent dir"))
        .expect("create wiki subdir");
    std::fs::write(&path, format!("---\n{frontmatter}---\n\n{body}\n"))
        .expect("write seeded wiki page");
}

/// Web API boots and serves the read-only surface with its token.
#[test]
fn daemon_health_web_api_and_auth() {
    let (_dir, root) = setup_test_project();
    let daemon = DaemonHandle::start(&root);

    // /api/health is deliberately exempt from auth.
    let (status, body) = daemon.raw("GET", "/api/health", &json!({}), None);
    assert_eq!(status, 200, "health should be public: {body}");
    let health: Value = serde_json::from_str(&body).expect("valid JSON");
    assert_eq!(health["success"], json!(true));

    // Read-only web API works with the web token.
    let resp = daemon.web_post("/api/initial", &json!({}));
    let body = parse_ok(&resp);
    assert!(
        body.get("graph_node_count").is_some(),
        "initial should report graph_node_count, got {body}"
    );

    let resp = daemon.web_post("/api/pages/list", &json!({}));
    let body = parse_ok(&resp);
    assert!(body.get("pages").is_some(), "pages/list missing pages: {body}");

    // Web API rejects missing/wrong tokens (AC: unauthenticated dispatch denied).
    let (status, _) = daemon.raw("POST", "/api/pages/list", &json!({}), None);
    assert_eq!(status, 401, "missing token should be rejected");
    let (status, _) = daemon.raw("POST", "/api/pages/list", &json!({}), Some("deadbeef"));
    assert_eq!(status, 401, "wrong token should be rejected");
}

/// Seed files directly on disk, then read them back through the web API:
/// search, graph, tasks, pages.get. Exercises the daemon's file watcher.
#[test]
fn web_api_end_to_end_flow() {
    let (_dir, root) = setup_test_project();
    let daemon = DaemonHandle::start(&root);

    seed_page(
        &root,
        "concepts/http-e2e-flow.md",
        "title: HTTP E2E Flow\ntype: concept\ntags: [e2e]\n",
        "Zirconium-quasar searchable term for the end to end flow test.",
    );
    seed_page(
        &root,
        "tasks/http-e2e-task.md",
        "title: HTTP E2E Task\ntype: task\nstatus: todo\n",
        "Track the zirco-quasar task across the daemon.",
    );

    // Wait for the watcher to index both pages into the graph.
    wait_until(&daemon, |d| {
        let (status, body) = d.web_post("/api/graph/stats", &json!({}));
        status == 200 && body["node_count"].as_i64().unwrap_or(0) >= 2
    });

    // Search finds seeded content through the web API.
    let resp = daemon.web_post(
        "/api/search/query",
        &json!({ "q": "zirconium-quasar", "type": "all", "limit": 10 }),
    );
    let body = parse_ok(&resp);
    let results = body["results"].as_array().expect("results array");
    assert!(
        !results.is_empty(),
        "search should return the seeded page, got {body}"
    );
    assert!(
        results.iter().any(|r| r["id"]
            .as_str()
            .is_some_and(|id| id.contains("http-e2e-flow"))),
        "search results should include http-e2e-flow, got {body}"
    );

    // Graph stats reflect both pages.
    let resp = daemon.web_post("/api/graph/stats", &json!({}));
    let body = parse_ok(&resp);
    let node_count = body["node_count"].as_i64().unwrap_or(0);
    assert!(
        node_count >= 2,
        "graph should contain the seeded pages, got node_count={node_count}"
    );

    // Tasks board renders the task page.
    let resp = daemon.web_post("/api/tasks/board", &json!({}));
    let body = parse_ok(&resp);
    assert!(
        body.get("columns").is_some() || body.get("todo").is_some(),
        "tasks board should return a board shape, got {body}"
    );

    // Page detail via web API. get_page returns raw content with
    // `meta: None` today, so assert on the content (frontmatter includes the
    // title) rather than the meta envelope.
    let resp = daemon.web_post(
        "/api/pages/get",
        &json!({ "id": "wiki:concepts:http-e2e-flow" }),
    );
    let body = parse_ok(&resp);
    assert_eq!(body["page"]["id"], json!("wiki:concepts:http-e2e-flow"));
    let content = body["page"]["content"].as_str().unwrap_or_default();
    assert!(
        content.contains("HTTP E2E Flow"),
        "page content should include the title frontmatter, got {body}"
    );
}

/// Without a bundled Angular build, the SPA fallback answers 404 instead of
/// serving a broken shell (the daemon still serves the API).
#[test]
fn spa_fallback_404_when_not_built() {
    let (_dir, root) = setup_test_project();
    let daemon = DaemonHandle::start(&root);

    let (status, body) = daemon.raw("GET", "/", &json!({}), None);
    assert_eq!(status, 404, "SPA not built should 404: {body}");
    let (status, _) = daemon.raw("GET", "/search", &json!({}), None);
    assert_eq!(status, 404, "SPA routes should 404 when not built");
}

fn cache_control(headers: &[(String, String)]) -> Option<String> {
    headers
        .iter()
        .find(|(name, _)| name == "cache-control")
        .map(|(_, value)| value.clone())
}

/// With a bundled Angular build present, `index.html` is served with
/// `Cache-Control: no-cache` (token freshness) while hashed assets get
/// `public, max-age=31536000, immutable` (Angular content-hashes filenames,
/// so long-lived caching is safe).
#[test]
fn spa_cache_control_headers() {
    let (_dir, root) = setup_test_project();
    let spa_dir = root.join("apps").join("wm-web").join("dist").join("browser");
    std::fs::create_dir_all(&spa_dir).expect("create fake spa dir");
    std::fs::write(
        spa_dir.join("index.html"),
        "<html><head></head><body>fake</body></html>",
    )
    .expect("write fake spa index");
    std::fs::write(
        spa_dir.join("main.4f2a1c.js"),
        "console.log('hashed');",
    )
    .expect("write hashed asset");

    let daemon = DaemonHandle::start(&root);

    let (status, headers, body) = daemon.get_headers("/");
    assert_eq!(status, 200, "SPA index should serve: {body}");
    assert_eq!(
        cache_control(&headers).as_deref(),
        Some("no-cache"),
        "index.html must be no-cache, got headers: {headers:?}"
    );

    let (status, headers, body) = daemon.get_headers("/index.html");
    assert_eq!(status, 200, "SPA index.html should serve: {body}");
    assert_eq!(
        cache_control(&headers).as_deref(),
        Some("no-cache"),
        "index.html must be no-cache, got headers: {headers:?}"
    );

    let (status, headers, body) = daemon.get_headers("/main.4f2a1c.js");
    assert_eq!(status, 200, "hashed asset should serve: {body}");
    assert_eq!(
        cache_control(&headers).as_deref(),
        Some("public, max-age=31536000, immutable"),
        "hashed assets must be immutable, got headers: {headers:?}"
    );

    let (status, headers, body) = daemon.get_headers("/assets/missing.js");
    assert_eq!(status, 200, "unknown SPA path falls back to index: {body}");
    assert_eq!(
        cache_control(&headers).as_deref(),
        Some("no-cache"),
        "SPA fallback (index) must be no-cache, got headers: {headers:?}"
    );
}
