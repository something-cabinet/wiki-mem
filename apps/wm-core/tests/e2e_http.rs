//! HTTP-API E2E suite against the live `wm-server` daemon (oracle D1).
//!
//! Tauri and the mock-server-backed CodeceptJS suite are gone (tasks 23138a /
//! 2c6a79 / b336c7). The daemon owns the engine + read-only web API + the
//! privileged MCP channel (`/api/mcp/*`); `wm mcp` is just a stdio→HTTP proxy.
//! This suite drives the real daemon over HTTP:
//!
//! - web API surface: health, initial, pages, search, graph, tasks, auth
//! - MCP channel: tools/list, tools/call, channel separation (web ≠ MCP token)
//! - SPA fallback behavior when no bundled frontend exists
//! - high-value MCP tool contracts (e2e.md TCs: link edge types, decision ADR
//!   fields, template create+get, unknown action/tool handling)

#[path = "helpers/setup.rs"]
mod setup;
use setup::setup_test_project;

#[path = "helpers/http_daemon.rs"]
mod daemon;
use daemon::DaemonHandle;

use serde_json::{json, Value};

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

fn mcp_ok(result: &Result<Value, (String, String)>) -> Value {
    match result {
        Ok(data) => data.clone(),
        Err((code, msg)) => panic!("MCP call failed [{code}]: {msg}"),
    }
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

/// The MCP channel lists tools and dispatches calls behind the MCP token only.
#[test]
fn mcp_channel_tools_list_call_and_auth() {
    let (_dir, root) = setup_test_project();
    let daemon = DaemonHandle::start(&root);

    let (status, body) = daemon.mcp_raw(
        "/api/mcp/tools/list",
        &json!({}),
        Some(&daemon.mcp_token),
    );
    assert_eq!(status, 200, "tools/list should be 2xx: {body}");
    let body: Value = serde_json::from_str(&body).expect("valid JSON");
    assert_eq!(body["success"], json!(true));
    let tools = body["data"]["tools"].as_array().expect("tools array");
    let names: Vec<&str> = tools
        .iter()
        .filter_map(|t| t["name"].as_str())
        .collect();
    for expected in ["wm_page", "wm_task", "wm_initial", "wm_search.query", "wm_decision"] {
        assert!(
            names.contains(&expected),
            "tools/list should include {expected}, got {names:?}"
        );
    }

    // Create + get round-trip over the MCP channel.
    let result = daemon.mcp_call_tool(
        "wm_page",
        json!({
            "action": "create",
            "path": "concepts/http-e2e-concept",
            "title": "HTTP E2E Concept",
            "content": "This page exercises the MCP channel of the wm-server daemon.",
        }),
    );
    let created = mcp_ok(&result);
    let id = created["id"].as_str().expect("created id").to_string();
    assert_eq!(created["type"], json!("concept"), "created type, got {created}");

    let result = daemon.mcp_call_tool(
        "wm_page",
        json!({ "action": "get", "id": id }),
    );
    let got = mcp_ok(&result);
    assert!(
        got["content"].as_str().unwrap_or_default().contains("MCP channel"),
        "get should return page content, got {got}"
    );

    // Channel separation: the web token must NOT authorize /api/mcp/*.
    let (status, _) = daemon.mcp_raw(
        "/api/mcp/tools/list",
        &json!({}),
        Some(&daemon.web_token),
    );
    assert_eq!(status, 401, "web token must not authorize the MCP channel");
    let (status, _) = daemon.mcp_raw("/api/mcp/tools/list", &json!({}), None);
    assert_eq!(status, 401, "no token must not authorize the MCP channel");
}

/// Unknown tools / actions fail cleanly through the MCP channel.
#[test]
fn unknown_tool_and_action_fail_cleanly() {
    let (_dir, root) = setup_test_project();
    let daemon = DaemonHandle::start(&root);

    // Unknown tool → METHOD_NOT_FOUND envelope (HTTP 200, success:false).
    let (status, body) = daemon.mcp_raw(
        "/api/mcp/tools/call",
        &json!({ "name": "wm_nonexistent", "arguments": {} }),
        Some(&daemon.mcp_token),
    );
    assert_eq!(status, 200, "tool-level errors are HTTP 200: {body}");
    let body: Value = serde_json::from_str(&body).expect("valid JSON");
    assert_eq!(body["success"], json!(false));
    assert_eq!(body["code"], json!("METHOD_NOT_FOUND"));

    // Unknown action on an existing tool → error envelope, never a crash.
    let err = daemon
        .mcp_call_tool("wm_page", json!({ "action": "bogus_action" }))
        .expect_err("unknown action must fail");
    let code = err.0.as_str();
    assert!(
        code == "SERDE_ERROR" || code == "INVALID_ACTION",
        "unknown action should return an error code, got {code:?}"
    );
}

/// Seed via the MCP channel, read it back through the web API: search, graph,
/// tasks, pages.get.
#[test]
fn web_api_end_to_end_flow() {
    let (_dir, root) = setup_test_project();
    let daemon = DaemonHandle::start(&root);

    mcp_ok(&daemon.mcp_call_tool(
        "wm_page",
        json!({
            "action": "create",
            "path": "concepts/http-e2e-flow",
            "title": "HTTP E2E Flow",
            "content": "Zirconium-quasar searchable term for the end to end flow test.",
            "tags": ["e2e"],
        }),
    ));
    mcp_ok(&daemon.mcp_call_tool(
        "wm_page",
        json!({
            "action": "create",
            "path": "tasks/http-e2e-task",
            "title": "HTTP E2E Task",
            "r#type": "task",
            "content": "Track the zirco-quasar task across the daemon.",
        }),
    ));

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

    // Page detail via web API. NOTE: get_page returns raw content with
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

/// #124 AC-2 regression through the daemon: the daemon (wm-server) boots
/// EngineState::new with NO file watcher, so after wm_task.update writes the
/// new status to disk, wm_task.get must still return the updated status
/// immediately — the write path refreshes the in-memory graph synchronously
/// (update_page_with_repo → handle_file_change). No wm_index_rebuild allowed.
#[test]
fn daemon_task_update_get_fresh_status_without_rebuild() {
    let (_dir, root) = setup_test_project();
    let daemon = DaemonHandle::start(&root);

    // Create a task (indexed into the in-memory graph by the create path).
    let created = mcp_ok(&daemon.mcp_call_tool(
        "wm_task",
        json!({
            "action": "create",
            "title": "Daemon Fresh Status Task",
            "status": "todo",
            "priority": "high",
        }),
    ));
    let id = created["id"].as_str().expect("created task id").to_string();
    assert_eq!(created["status"], json!("todo"), "created status, got {created}");

    // Update the status — no index rebuild afterwards.
    let updated = mcp_ok(&daemon.mcp_call_tool(
        "wm_task",
        json!({ "action": "update", "id": id, "status": "in-progress" }),
    ));
    assert_eq!(updated["status"], json!("updated"), "update failed: {updated}");

    // get must return the updated status immediately (AC-2, daemon context).
    let got = mcp_ok(&daemon.mcp_call_tool(
        "wm_task",
        json!({ "action": "get", "id": id }),
    ));
    assert_eq!(
        got["status"].as_str(),
        Some("in-progress"),
        "daemon wm_task.get must return the updated status immediately (no rebuild), got: {got}"
    );

    // list must also reflect it (graph-backed, not just the single-node read).
    let listed = mcp_ok(&daemon.mcp_call_tool(
        "wm_task",
        json!({ "action": "list" }),
    ));
    let tasks = listed["tasks"].as_array().expect("tasks array");
    let mine = tasks
        .iter()
        .find(|t| t["id"].as_str() == Some(id.as_str()))
        .expect("created task present in list");
    assert_eq!(
        mine["status"].as_str(),
        Some("in-progress"),
        "list must reflect the updated status, got: {listed}"
    );
}

/// TC-2.5: wm_page.link across every built-in edge type, plus unlink.
#[test]
fn page_link_supports_all_builtin_edge_types() {
    let (_dir, root) = setup_test_project();
    let daemon = DaemonHandle::start(&root);

    mcp_ok(&daemon.mcp_call_tool(
        "wm_page",
        json!({
            "action": "create",
            "path": "concepts/link-src",
            "title": "Link Source",
            "content": "Source page for edge type coverage.",
        }),
    ));
    mcp_ok(&daemon.mcp_call_tool(
        "wm_page",
        json!({
            "action": "create",
            "path": "concepts/link-dst",
            "title": "Link Destination",
            "content": "Destination page for edge type coverage.",
        }),
    ));

    // wm-engine defines 9 built-in edge types (the original spec said 17; the
    // engine consolidated to 9 + EdgeType::Custom — see e2e.md note).
    let builtin: &[&str] = &[
        "extends",
        "implements",
        "example_of",
        "part_of",
        "relates_to",
        "supersedes",
        "depends_on",
        "answers",
        "references",
    ];
    for edge in builtin {
        let result = daemon.mcp_call_tool(
            "wm_page",
            json!({
                "action": "link",
                "id": "wiki:concepts:link-src",
                "target": "wiki:concepts:link-dst",
                "edge_type": edge,
            }),
        );
        let linked = mcp_ok(&result);
        assert_eq!(linked["status"], json!("linked"), "link {edge} failed");
        assert_eq!(linked["type"], json!(edge));

        // The edge must be persisted in the source file's frontmatter with the
        // requested type and target (link REPLACES relates_to, so the file
        // holds exactly this one edge after each call).
        let file = root
            .join(".wm")
            .join("wiki")
            .join("concepts")
            .join("link-src.md");
        let raw = std::fs::read_to_string(&file).expect("link-src.md on disk");
        let edge_line = format!("- {{type: {edge}, target: wiki:concepts:link-dst}}");
        assert!(
            raw.contains(&edge_line),
            "file should persist {{type: {edge}, target: wiki:concepts:link-dst}} after link, got:\n{raw}"
        );
        assert!(!raw.contains("{}"), "no empty frontmatter block after link: {raw}");
    }

    // Unlink removes the edge from disk and reports `unlinked`.
    let result = daemon.mcp_call_tool(
        "wm_page",
        json!({
            "action": "unlink",
            "id": "wiki:concepts:link-src",
            "target": "wiki:concepts:link-dst",
        }),
    );
    let unlinked = mcp_ok(&result);
    assert_eq!(unlinked["status"], json!("unlinked"));

    let file = root
        .join(".wm")
        .join("wiki")
        .join("concepts")
        .join("link-src.md");
    let raw = std::fs::read_to_string(&file).expect("link-src.md on disk");
    assert!(
        !raw.contains("wiki:concepts:link-dst"),
        "unlink must remove the edge from the file, got:\n{raw}"
    );
}

/// TC-2.11: wm_decision.create with ADR fields serializes them; the round trip
/// via get works after an index rebuild.
///
/// NOTE (test rot): `wm_decision.create` does NOT call `handle_file_change`
/// (unlike `wm_page.create`), so the page lands on disk but is absent from the
/// in-memory graph until the watcher/index refresh. The test drives a rebuild
/// to make the round trip deterministic and to pin this behavior.
#[test]
fn decision_create_with_adr_fields() {
    let (_dir, root) = setup_test_project();
    let daemon = DaemonHandle::start(&root);

    let result = daemon.mcp_call_tool(
        "wm_decision",
        json!({
            "action": "create",
            "id": "wiki:decisions:http-e2e-decision",
            "title": "HTTP E2E Decision",
            "context": "The daemon now owns the engine and the HTTP API.",
            "options": ["A", "B"],
            "rationale": "Single daemon simplifies discovery and token lifecycle.",
            "outcome": "Accepted",
        }),
    );
    let created = mcp_ok(&result);
    assert_eq!(created["title"], json!("HTTP E2E Decision"));
    assert_eq!(created["status"], json!("draft"));

    // ADR fields must be persisted in the frontmatter on disk.
    let file = root.join(".wm").join("wiki").join("decisions").join("http-e2e-decision.md");
    let raw = std::fs::read_to_string(&file).expect("decision file should exist on disk");
    for field in ["context:", "rationale:", "options:", "outcome:", "daemon now owns"] {
        assert!(
            raw.contains(field),
            "decision file should contain {field:?}, got:\n{raw}"
        );
    }

    // Round trip via get after a graph rebuild (skip embeddings: no model).
    mcp_ok(&daemon.mcp_call_tool(
        "wm_index_rebuild",
        json!({ "skip_embed": true }),
    ));
    let result = daemon.mcp_call_tool(
        "wm_decision",
        json!({ "action": "get", "id": "wiki:decisions:http-e2e-decision" }),
    );
    let got = mcp_ok(&result);
    assert_eq!(got["title"], json!("HTTP E2E Decision"));
    assert!(
        got["context"]
            .as_str()
            .is_some_and(|c| c.contains("daemon")),
        "decision get should return the ADR context, got {got}"
    );
}

/// TC-2.12: wm_template.create persists a template (run removed in #24).
#[test]
fn template_create_and_get() {
    let (_dir, root) = setup_test_project();
    let daemon = DaemonHandle::start(&root);

    mcp_ok(&daemon.mcp_call_tool(
        "wm_template",
        json!({
            "action": "create",
            "name": "http-e2e-template",
            "description": "HTTP E2E template",
            "content": "Hello {{name}} from the wm-server daemon.",
        }),
    ));

    let result = daemon.mcp_call_tool(
        "wm_template",
        json!({
            "action": "get",
            "name": "http-e2e-template",
        }),
    );
    let out = mcp_ok(&result);
    let text = serde_json::to_string(&out).unwrap_or_default();
    assert!(
        text.contains("Hello {{name}}"),
        "template get should return the stored content, got {out}"
    );

    let listed = daemon.mcp_call_tool("wm_template", json!({ "action": "list" }));
    let out = mcp_ok(&listed);
    let text = serde_json::to_string(&out).unwrap_or_default();
    assert!(
        text.contains("http-e2e-template"),
        "template list should include the created template, got {out}"
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

/// TC-1.3: wm_doc no longer exposes a `meta_mut` action (removed tool — zero
/// references in src). The remaining doc CRUD actions must still round-trip:
/// create → get → update → get → delete → get(not found).
#[test]
fn wm_doc_crud_roundtrip_without_meta_mut() {
    let (_dir, root) = setup_test_project();
    let daemon = DaemonHandle::start(&root);

    // The wm_doc schema must NOT advertise a meta_mut action.
    let (status, body) = daemon.mcp_raw(
        "/api/mcp/tools/list",
        &json!({}),
        Some(&daemon.mcp_token),
    );
    assert_eq!(status, 200, "tools/list should be 2xx: {body}");
    let body: Value = serde_json::from_str(&body).expect("valid JSON");
    let tools = body["data"]["tools"].as_array().expect("tools array");
    let doc_tool = tools
        .iter()
        .find(|t| t["name"].as_str() == Some("wm_doc"))
        .expect("wm_doc should be listed");
    let schema_text = serde_json::to_string(&doc_tool["inputSchema"]).unwrap_or_default();
    assert!(
        !schema_text.contains("meta_mut"),
        "wm_doc must not expose meta_mut, got: {schema_text}"
    );

    // create
    let created = mcp_ok(&daemon.mcp_call_tool(
        "wm_doc",
        json!({
            "action": "create",
            "path": "concepts/http-e2e-doc",
            "title": "HTTP E2E Doc",
            "content": "Body of the e2e doc.",
        }),
    ));
    assert_eq!(created["status"], json!("created"), "create failed: {created}");
    let file = root.join(".wm").join("wiki").join("concepts").join("http-e2e-doc.md");
    assert!(file.exists(), "doc file should exist on disk");

    // get
    let got = mcp_ok(&daemon.mcp_call_tool(
        "wm_doc",
        json!({ "action": "get", "path": "concepts/http-e2e-doc" }),
    ));
    assert_eq!(got["title"], json!("HTTP E2E Doc"));
    assert!(
        got["content"].as_str().unwrap_or_default().contains("Body of the e2e doc."),
        "get should return the doc body, got {got}"
    );

    // update
    let upd = mcp_ok(&daemon.mcp_call_tool(
        "wm_doc",
        json!({
            "action": "update",
            "path": "concepts/http-e2e-doc",
            "title": "HTTP E2E Doc Updated",
            "content": "Updated body.",
        }),
    ));
    assert_eq!(upd["status"], json!("updated"), "update failed: {upd}");

    // get reflects the update
    let got = mcp_ok(&daemon.mcp_call_tool(
        "wm_doc",
        json!({ "action": "get", "path": "concepts/http-e2e-doc" }),
    ));
    assert_eq!(got["title"], json!("HTTP E2E Doc Updated"));
    assert!(
        got["content"].as_str().unwrap_or_default().contains("Updated body."),
        "get should return the updated body, got {got}"
    );

    // delete
    let del = mcp_ok(&daemon.mcp_call_tool(
        "wm_doc",
        json!({ "action": "delete", "path": "concepts/http-e2e-doc" }),
    ));
    assert_eq!(del["status"], json!("deleted"), "delete failed: {del}");
    assert!(!file.exists(), "doc file should be removed after delete");

    // get after delete → not found error envelope
    let err = daemon
        .mcp_call_tool("wm_doc", json!({ "action": "get", "path": "concepts/http-e2e-doc" }))
        .expect_err("get after delete must fail");
    assert!(
        err.0 == "NOT_FOUND" || err.0 == "TOOL_ERROR",
        "deleted doc get should error, got code {}: {}",
        err.0,
        err.1
    );
}

/// TC-2.9: wm_memory.promote moves an entry from the project layer to the
/// global layer.
///
/// The global layer is HOME-based (`$HOME/.wm/wiki/memory/<slug>.md`,
/// mirroring the project `.wm/wiki/memory/` layout) — the daemon is spawned
/// with a redirected HOME so the write never touches the real home directory.
/// promote reads the project entry from disk (no graph dependency) and copies
/// it; the project copy is kept.
#[test]
fn memory_promote_moves_entry_to_global_layer() {
    let (_dir, root) = setup_test_project();
    let home_dir = tempfile::tempdir().expect("temp HOME for global layer");
    let home = home_dir.path().to_string_lossy().to_string();
    let daemon = DaemonHandle::start_with_env(&root, &[("HOME", &home)]);

    // Add a memory entry in the project layer.
    let added = mcp_ok(&daemon.mcp_call_tool(
        "wm_memory",
        json!({
            "action": "add",
            "title": "HTTP E2E Promote Me",
            "content": "Memory entry that will be promoted to the global layer.",
            "tags": ["e2e"],
        }),
    ));
    let id = added["id"].as_str().expect("memory id").to_string();
    assert!(id.contains("http-e2e-promote-me"), "memory id: {id}");

    // Promote project → global.
    let promoted = mcp_ok(&daemon.mcp_call_tool(
        "wm_memory",
        json!({ "action": "promote", "id": id }),
    ));
    assert_eq!(promoted["status"], json!("promoted"), "promote failed: {promoted}");
    assert_eq!(promoted["source"], json!("project"));
    assert_eq!(promoted["target"], json!("global"));

    // The promoted copy lands under the redirected HOME, never the real one.
    let global_file = std::path::PathBuf::from(&home)
        .join(".wm")
        .join("wiki")
        .join("memory")
        .join("http-e2e-promote-me.md");
    assert!(
        global_file.exists(),
        "promoted file should exist at {} (HOME redirect)",
        global_file.display()
    );
    let raw = std::fs::read_to_string(&global_file).expect("read promoted file");
    assert!(
        raw.contains("promoted to the global layer"),
        "promoted file should carry the memory content, got: {raw}"
    );

    // Project entry is untouched (promote copies, it does not remove).
    let project_file = root
        .join(".wm")
        .join("wiki")
        .join("memory")
        .join("http-e2e-promote-me.md");
    assert!(
        project_file.exists(),
        "project memory entry should still exist after promote"
    );
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
