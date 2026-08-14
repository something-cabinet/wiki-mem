//! MCP tool-contract tests.
//!
//! The bulk of the suite dispatches through the real `ToolRegistry`
//! in-process — tempdir project, `register_all_tools`, `dispatch_async` — so
//! the full handler pipeline (schema deserialization → confinement → audit) is
//! covered without spawning subprocesses. A thin stdio seam keeps the actual
//! `wm-cli mcp` JSON-RPC transport honest: initialize handshake, tools/list,
//! and one tools/call round trip.

#[path = "helpers/mcp.rs"]
mod helpers;
use helpers::MCPClient;

#[path = "helpers/inproc.rs"]
mod inproc;
use inproc::{call, call_err, call_ok, setup_in_process};

use serde_json::json;
use wm_core::mcp::transport::ToolRegistry;

async fn rebuild(registry: &ToolRegistry) -> serde_json::Value {
    call_ok(registry, "wm_index_rebuild", json!({ "skip_embed": true })).await
}

fn setup_stdio() -> (tempfile::TempDir, MCPClient) {
    let (dir, root) = inproc::setup::setup_test_project();
    let client = MCPClient::start(&root);
    (dir, client)
}

// ---------------------------------------------------------------------------
// Transport seam (stdio) — the JSON-RPC layer needs exactly this much
// ---------------------------------------------------------------------------

/// `wm mcp` must complete the JSON-RPC initialize handshake with the
/// protocol version, server identity and session instructions.
#[test]
fn stdio_initialize_handshake() {
    let (_dir, mut client) = setup_stdio();
    let resp = client.initialize().expect("initialize failed");
    let result = resp.get("result").expect("no result");
    assert_eq!(
        result.get("protocolVersion").and_then(|v| v.as_str()),
        Some("2024-11-05")
    );
    assert_eq!(
        result
            .get("serverInfo")
            .and_then(|r| r.get("name"))
            .and_then(|v| v.as_str()),
        Some("wm-engine")
    );
    assert!(result
        .get("instructions")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .contains("wm_initial"));
}

/// tools/list over stdio must serialize the full registered tool surface.
#[test]
fn stdio_tools_list() {
    let (_dir, mut client) = setup_stdio();
    client.initialize().expect("initialize");
    let tools = client.list_tools().expect("list_tools");
    assert!(tools.len() >= 30, "expected 30+ tools, got {}", tools.len());
    for tool in [
        "wm_initial",
        "wm_help",
        "wm_page",
        "wm_search.query",
        "wm_graph.stats",
        "wm_task",
        "wm_code.search",
    ] {
        assert!(
            tools.iter().any(|t| t == tool),
            "missing essential tool: {tool}"
        );
    }
}

/// A real tools/call round trip through the stdio transport: the JSON-RPC
/// response must carry the matching id, no protocol error, and the tool's
/// result payload.
#[test]
fn stdio_call_round_trip() {
    let (_dir, mut client) = setup_stdio();
    client.initialize().expect("initialize");
    let resp = client
        .send_request_raw(
            "tools/call",
            json!({ "name": "wm_page", "arguments": { "action": "list" } }),
        )
        .expect("tools/call via stdio");
    assert!(resp.get("error").is_none(), "protocol error in: {resp}");
    let result = resp.get("result").expect("tools/call result");
    assert_eq!(result.get("isError"), Some(&serde_json::Value::Bool(false)));
    let text = result
        .get("content")
        .and_then(|c| c.as_array())
        .and_then(|a| a.first())
        .and_then(|c| c.get("text"))
        .and_then(|t| t.as_str())
        .expect("result content text");
    let payload: serde_json::Value = serde_json::from_str(text).expect("tool payload must be JSON");
    assert!(payload.get("pages").is_some());
    assert!(payload.get("total").is_some());

    // Tool-level errors must surface through the transport as isError.
    let err = client
        .call_tool(
            "wm_page",
            json!({ "action": "get", "id": "nonexistent:id" }),
        )
        .expect_err("unknown page must error through the transport");
    assert!(err.contains("not found"), "got: {err}");
}

// ---------------------------------------------------------------------------
// Bootstrap — session context, help, project, model, log
// ---------------------------------------------------------------------------

/// wm_initial injects project context and graph/section counts.
#[tokio::test(flavor = "multi_thread")]
async fn wm_initial_reports_active_project() {
    let ((_dir, _root, _engine, registry), _cwd) = setup_in_process().await;
    let out = call_ok(&registry, "wm_initial", json!({})).await;
    assert_eq!(out.get("project").and_then(|v| v.as_str()), Some("active"));
    assert!(out.get("graph_nodes").is_some());
    assert!(out.get("graph_edges").is_some());
    assert!(out.get("search_modes_available").is_some());
}

#[tokio::test(flavor = "multi_thread")]
async fn wm_help_lists_and_filters_tools() {
    let ((_dir, _root, _engine, registry), _cwd) = setup_in_process().await;
    let all = call_ok(&registry, "wm_help", json!({})).await;
    let tools = all
        .get("available_tools")
        .and_then(|v| v.as_array())
        .expect("available_tools");
    assert!(tools.len() >= 30, "expected 30+ tools, got {}", tools.len());
    let filtered = call_ok(&registry, "wm_help", json!({ "q": "search" })).await;
    let filtered_tools = filtered
        .get("available_tools")
        .and_then(|v| v.as_array())
        .expect("filtered available_tools");
    assert!(!filtered_tools.is_empty(), "expected search-related tools");
}

#[tokio::test(flavor = "multi_thread")]
async fn wm_project_reports_active_and_detectable() {
    let ((_dir, _root, _engine, registry), _cwd) = setup_in_process().await;
    let status = call_ok(&registry, "wm_project.status", json!({})).await;
    assert_eq!(
        status.get("project").and_then(|v| v.as_str()),
        Some("active")
    );
    let detect = call_ok(&registry, "wm_project.detect", json!({})).await;
    assert_eq!(
        detect.get("project").and_then(|v| v.as_str()),
        Some("detected")
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn wm_model_list_reports_active_and_available() {
    let ((_dir, _root, _engine, registry), _cwd) = setup_in_process().await;
    let out = call_ok(&registry, "wm_model", json!({ "action": "list" })).await;
    assert!(out.get("active_model").and_then(|v| v.as_str()).is_some());
    assert!(out.get("models").and_then(|v| v.as_array()).is_some());
}

#[tokio::test(flavor = "multi_thread")]
async fn wm_log_recent_returns_entries() {
    let ((_dir, _root, _engine, registry), _cwd) = setup_in_process().await;
    let out = call_ok(&registry, "wm_log.recent", json!({})).await;
    assert!(out.get("entries").is_some());
    assert!(out.get("total").is_some());
}

// ---------------------------------------------------------------------------
// Pages
// ---------------------------------------------------------------------------

async fn page_create(registry: &ToolRegistry, path: &str, title: &str, content: &str) -> String {
    let out = call_ok(
        registry,
        "wm_page",
        json!({ "action": "create", "path": path, "title": title, "content": content }),
    )
    .await;
    out.get("id")
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_string()
}

#[tokio::test(flavor = "multi_thread")]
async fn page_create_get_and_list_round_trip() {
    let ((_dir, _root, _engine, registry), _cwd) = setup_in_process().await;
    let id = page_create(
        &registry,
        "concepts/round-trip",
        "Round Trip",
        "# Round Trip\n\nBody.",
    )
    .await;
    assert!(
        id.contains("round-trip"),
        "expected page id to carry the path, got {id}"
    );

    let got = call_ok(&registry, "wm_page", json!({ "action": "get", "id": id })).await;
    assert_eq!(got.get("id").and_then(|v| v.as_str()), Some(id.as_str()));
    assert!(got
        .get("content")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .contains("Round Trip"));

    let listed = call_ok(&registry, "wm_page", json!({ "action": "list" })).await;
    assert_eq!(listed.get("total").and_then(|v| v.as_u64()), Some(1));
    let pages = listed
        .get("pages")
        .and_then(|v| v.as_array())
        .expect("pages");
    assert_eq!(pages.len(), 1);
    assert!(pages[0].get("id").and_then(|v| v.as_str()).is_some());
}

/// A created page must carry `id:` in its frontmatter on disk.
#[tokio::test(flavor = "multi_thread")]
async fn page_create_emits_id_frontmatter() {
    let ((_dir, root, _engine, registry), _cwd) = setup_in_process().await;
    page_create(
        &registry,
        "concepts/id-frontmatter",
        "ID Frontmatter",
        "Body",
    )
    .await;
    let content = std::fs::read_to_string(root.join(".wm/wiki/concepts/id-frontmatter.md"))
        .expect("created page on disk");
    assert!(
        content.contains("id: \"wiki:concepts:id-frontmatter\""),
        "frontmatter must carry the canonical id (double-quoted), got:\n{content}"
    );
}

/// Update must accept the canonical `id` parameter and persist on disk.
#[tokio::test(flavor = "multi_thread")]
async fn page_update_uses_id_parameter() {
    let ((_dir, root, _engine, registry), _cwd) = setup_in_process().await;
    let id = page_create(&registry, "regression/id-param", "ID Param", "Body").await;
    let out = call_ok(
        &registry,
        "wm_page",
        json!({ "action": "update", "id": id, "title": "Updated via id" }),
    )
    .await;
    assert_eq!(out.get("status").and_then(|v| v.as_str()), Some("updated"));
    let content = std::fs::read_to_string(root.join(".wm/wiki/regression/id-param.md"))
        .expect("updated page on disk");
    assert!(
        content.contains("title: Updated via id"),
        "title must persist, got:\n{content}"
    );
}

/// Extra frontmatter fields must round-trip losslessly through update.
#[tokio::test(flavor = "multi_thread")]
async fn page_update_extra_frontmatter_persists() {
    let ((_dir, root, _engine, registry), _cwd) = setup_in_process().await;
    let id = page_create(&registry, "regression/extra-fm", "Extra FM", "Body.").await;
    let out = call_ok(
        &registry,
        "wm_page",
        json!({
            "action": "update",
            "id": id,
            "type": "pattern",
            "extra_frontmatter": {
                "knowns_id": "legacy-007",
                "confidence": "high",
                "aliases": ["alpha", "beta"],
                "nested": {"depth": 2, "ok": true},
            },
        }),
    )
    .await;
    assert_eq!(out.get("status").and_then(|v| v.as_str()), Some("updated"));

    let content = std::fs::read_to_string(root.join(".wm/wiki/regression/extra-fm.md"))
        .expect("updated page on disk");
    for needle in [
        "knowns_id: legacy-007",
        "type: pattern",
        "confidence: high",
        "- alpha",
        "depth: 2",
        "Body.",
    ] {
        assert!(
            content.contains(needle),
            "missing {needle:?} in:\n{content}"
        );
    }
    let (fm, _body) = wm_core::parser::extract_frontmatter(&content);
    let fm = fm.expect("frontmatter parses");
    assert_eq!(fm.page_type.as_deref(), Some("pattern"));
    assert_eq!(fm.title.as_deref(), Some("Extra FM"));
}

/// AC-1: wm_doc.create with `type` must persist `type:` into the YAML
/// frontmatter on disk (regression for GitHub issue #126).
#[tokio::test(flavor = "multi_thread")]
async fn doc_create_persists_type_frontmatter() {
    let ((_dir, root, _engine, registry), _cwd) = setup_in_process().await;
    call_ok(
        &registry,
        "wm_doc",
        json!({
            "action": "create",
            "path": "specs/repro-x",
            "title": "X",
            "type": "spec",
            "content": "Body content.",
        }),
    )
    .await;
    let content = std::fs::read_to_string(root.join(".wm/wiki/specs/repro-x.md"))
        .expect("created doc on disk");
    assert!(
        content.contains("type: spec"),
        "frontmatter must contain `type: spec`, got:\n{content}"
    );
    assert!(
        content.contains("title: X"),
        "title must persist, got:\n{content}"
    );
}

/// FR-3: wm_doc.create without `type` derives it from the path directory,
/// matching wm_page's default (no untyped pages).
#[tokio::test(flavor = "multi_thread")]
async fn doc_create_derives_type_from_path_dir() {
    let ((_dir, root, _engine, registry), _cwd) = setup_in_process().await;
    call_ok(
        &registry,
        "wm_doc",
        json!({
            "action": "create",
            "path": "concepts/derived-type",
            "title": "Derived Type",
            "content": "Body.",
        }),
    )
    .await;
    let content = std::fs::read_to_string(root.join(".wm/wiki/concepts/derived-type.md"))
        .expect("created doc on disk");
    assert!(
        content.contains("type: concept"),
        "path dir must derive type, got:\n{content}"
    );
}

/// AC-2: wm_doc.update with `type` retypes the frontmatter while preserving
/// the existing title and body.
#[tokio::test(flavor = "multi_thread")]
async fn doc_update_retypes_preserving_title_and_body() {
    let ((_dir, root, _engine, registry), _cwd) = setup_in_process().await;
    call_ok(
        &registry,
        "wm_doc",
        json!({
            "action": "create",
            "path": "specs/repro-x",
            "title": "X",
            "type": "spec",
            "content": "Original body.",
        }),
    )
    .await;
    let out = call_ok(
        &registry,
        "wm_doc",
        json!({ "action": "update", "path": "specs/repro-x", "type": "howto" }),
    )
    .await;
    assert_eq!(out.get("status").and_then(|v| v.as_str()), Some("updated"));
    let content = std::fs::read_to_string(root.join(".wm/wiki/specs/repro-x.md"))
        .expect("updated doc on disk");
    assert!(
        content.contains("type: howto"),
        "type must be retyped, got:\n{content}"
    );
    assert!(
        !content.contains("type: spec"),
        "old type must be replaced, got:\n{content}"
    );
    assert!(
        content.contains("title: X"),
        "title must be preserved, got:\n{content}"
    );
    assert!(
        content.contains("Original body."),
        "body must be preserved, got:\n{content}"
    );
}

/// AC-3: wm_doc.update with `tags` persists them inline and preserves the
/// existing `type` when it is not provided.
#[tokio::test(flavor = "multi_thread")]
async fn doc_update_persists_tags_and_preserves_type() {
    let ((_dir, root, _engine, registry), _cwd) = setup_in_process().await;
    call_ok(
        &registry,
        "wm_doc",
        json!({
            "action": "create",
            "path": "specs/repro-y",
            "title": "Y",
            "type": "spec",
            "content": "Keep me.",
        }),
    )
    .await;
    let out = call_ok(
        &registry,
        "wm_doc",
        json!({ "action": "update", "path": "specs/repro-y", "tags": ["a", "b"] }),
    )
    .await;
    assert_eq!(out.get("status").and_then(|v| v.as_str()), Some("updated"));
    let content = std::fs::read_to_string(root.join(".wm/wiki/specs/repro-y.md"))
        .expect("updated doc on disk");
    assert!(
        content.contains("tags: [a, b]"),
        "tags must persist inline, got:\n{content}"
    );
    assert!(
        content.contains("type: spec"),
        "existing type must be preserved, got:\n{content}"
    );
    assert!(
        content.contains("title: Y"),
        "title must be preserved, got:\n{content}"
    );
    assert!(
        content.contains("Keep me."),
        "body must be preserved, got:\n{content}"
    );
}

/// FR-3 (via the wm_doc alias): wm_doc.get routes through the page path, so
/// files not registered in the graph — e.g. pages written by the retired
/// doc.rs writer — are still readable via the filesystem fallback.
#[tokio::test(flavor = "multi_thread")]
async fn doc_get_reads_legacy_file_via_page_path() {
    let ((_dir, root, _engine, registry), _cwd) = setup_in_process().await;
    std::fs::create_dir_all(root.join(".wm/wiki/howto")).expect("create howto dir");
    std::fs::write(
        root.join(".wm/wiki/howto/legacy.md"),
        "---\ntitle: Legacy\ntype: howto\n---\n\nBody.",
    )
    .expect("write legacy file");
    let out = call_ok(
        &registry,
        "wm_doc",
        json!({ "action": "get", "path": "howto/legacy" }),
    )
    .await;
    let content = out.get("content").and_then(|v| v.as_str()).unwrap_or("");
    assert!(
        content.contains("Legacy"),
        "legacy file must be readable via wm_doc.get, got: {out}"
    );
    assert!(
        content.contains("type: howto"),
        "frontmatter must be intact, got: {out}"
    );
}

/// wm_page.get must accept the canonical `wiki:`-prefixed id.
#[tokio::test(flavor = "multi_thread")]
async fn page_get_by_canonical_id() {
    let ((_dir, _root, _engine, registry), _cwd) = setup_in_process().await;
    page_create(
        &registry,
        "concepts/get-by-id",
        "Get By ID",
        "Body via canonical id.",
    )
    .await;
    let out = call_ok(
        &registry,
        "wm_page",
        json!({ "action": "get", "id": "wiki:concepts:get-by-id" }),
    )
    .await;
    assert_eq!(
        out.get("id").and_then(|v| v.as_str()),
        Some("wiki:concepts:get-by-id")
    );
    assert!(out
        .get("content")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .contains("canonical id"));
}

/// An invalid action must be rejected by schema deserialization.
#[tokio::test(flavor = "multi_thread")]
async fn page_invalid_action_is_rejected() {
    let ((_dir, _root, _engine, registry), _cwd) = setup_in_process().await;
    let err = call_err(&registry, "wm_page", json!({ "action": "fly" })).await;
    assert!(
        err.message.contains("fly")
            || err.message.contains("action")
            || err.message.contains("invalid"),
        "expected an action validation error, got: {}",
        err.message
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn page_missing_required_fields_are_rejected() {
    let ((_dir, _root, _engine, registry), _cwd) = setup_in_process().await;
    let err = call_err(&registry, "wm_page", json!({})).await;
    assert!(
        err.message.contains("required")
            || err.message.contains("missing")
            || err.message.contains("action"),
        "expected a missing-field error, got: {}",
        err.message
    );
    let err = call_err(&registry, "wm_page", json!({ "action": "get" })).await;
    assert!(
        err.message.contains("required")
            || err.message.contains("missing")
            || err.message.contains("id"),
        "expected a missing-id error, got: {}",
        err.message
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn page_get_unknown_id_returns_not_found() {
    let ((_dir, _root, _engine, registry), _cwd) = setup_in_process().await;
    let err = call_err(
        &registry,
        "wm_page",
        json!({ "action": "get", "id": "nonexistent:id" }),
    )
    .await;
    assert!(err.message.contains("not found"), "got: {}", err.message);
}

// ---------------------------------------------------------------------------
// Search
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread")]
async fn search_query_echoes_and_returns_results() {
    let ((_dir, _root, _engine, registry), _cwd) = setup_in_process().await;
    let out = call_ok(
        &registry,
        "wm_search.query",
        json!({ "q": "test", "limit": 5 }),
    )
    .await;
    assert_eq!(out.get("query").and_then(|v| v.as_str()), Some("test"));
    assert!(out.get("results").and_then(|v| v.as_array()).is_some());
    assert!(out.get("total").is_some());
}

#[tokio::test(flavor = "multi_thread")]
async fn search_retrieve_assembles_context() {
    let ((_dir, _root, _engine, registry), _cwd) = setup_in_process().await;
    let out = call_ok(
        &registry,
        "wm_search.retrieve",
        json!({ "q": "test", "token_budget": 4096 }),
    )
    .await;
    assert!(out.get("tokens_used").is_some());
    assert!(out.get("context").is_some());
}

#[tokio::test(flavor = "multi_thread")]
async fn search_type_filter_returns_only_requested_type() {
    let ((_dir, _root, _engine, registry), _cwd) = setup_in_process().await;
    page_create(
        &registry,
        "concepts/type-filter",
        "Type Filter",
        "Type filter body.",
    )
    .await;
    rebuild(&registry).await;
    let out = call_ok(
        &registry,
        "wm_search.query",
        json!({ "q": "Type Filter", "type": "page", "limit": 10 }),
    )
    .await;
    let results = out
        .get("results")
        .and_then(|v| v.as_array())
        .expect("results");
    assert!(!results.is_empty(), "expected results for type=page");
    for r in results {
        assert_eq!(r.get("type").and_then(|v| v.as_str()), Some("page"));
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn search_hybrid_falls_back_without_embedder() {
    let ((_dir, _root, _engine, registry), _cwd) = setup_in_process().await;
    page_create(
        &registry,
        "concepts/hybrid-fallback",
        "Hybrid Fallback",
        "Fallback body.",
    )
    .await;
    rebuild(&registry).await;
    let out = call_ok(
        &registry,
        "wm_search.query",
        json!({ "q": "Hybrid Fallback", "mode": "hybrid", "limit": 5 }),
    )
    .await;
    let mode = out.get("mode").and_then(|v| v.as_str()).unwrap_or("");
    assert!(
        mode == "hybrid" || mode == "keyword",
        "expected hybrid or keyword mode, got '{mode}'"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn search_query_missing_q_is_rejected() {
    let ((_dir, _root, _engine, registry), _cwd) = setup_in_process().await;
    let err = call_err(&registry, "wm_search.query", json!({})).await;
    assert!(
        err.message.contains("required")
            || err.message.contains("missing")
            || err.message.contains("q"),
        "expected a missing-q error, got: {}",
        err.message
    );
}

// ---------------------------------------------------------------------------
// Graph, lint, validate, index
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread")]
async fn graph_stats_reports_nodes_and_edges() {
    let ((_dir, _root, _engine, registry), _cwd) = setup_in_process().await;
    let out = call_ok(&registry, "wm_graph.stats", json!({})).await;
    assert!(out.get("nodes").is_some());
    assert!(out.get("edges").is_some());
}

#[tokio::test(flavor = "multi_thread")]
async fn lint_check_reports_issues_and_total() {
    let ((_dir, _root, _engine, registry), _cwd) = setup_in_process().await;
    let out = call_ok(&registry, "wm_lint.check", json!({})).await;
    assert!(out.get("issues").is_some());
    assert!(out.get("total").is_some());
}

/// Lint must flag a page missing `id:` and pass once it is present.
#[tokio::test(flavor = "multi_thread")]
async fn lint_check_catches_missing_id() {
    let ((_dir, root, _engine, registry), _cwd) = setup_in_process().await;
    std::fs::write(
        root.join(".wm/wiki/concepts/no-id-test.md"),
        "---\ntitle: No ID Test\ntype: concept\n---\n\nBody here.\n",
    )
    .expect("write page without id");
    call_ok(&registry, "wm_index_rebuild", json!({})).await;

    let out = call_ok(&registry, "wm_lint.check", json!({})).await;
    let issues = out
        .get("issues")
        .and_then(|v| v.as_array())
        .expect("issues");
    let missing: Vec<_> = issues
        .iter()
        .filter(|i| i.get("type").and_then(|v| v.as_str()) == Some("missing_id"))
        .collect();
    assert_eq!(
        missing.len(),
        1,
        "expected exactly 1 missing_id issue, got {issues:?}"
    );

    std::fs::write(
        root.join(".wm/wiki/concepts/no-id-test.md"),
        "---\ntitle: No ID Test\ntype: concept\nid: wiki:concepts:no-id-test\n---\n\nBody here.\n",
    )
    .expect("write page with id");
    call_ok(&registry, "wm_index_rebuild", json!({})).await;
    let out = call_ok(&registry, "wm_lint.check", json!({})).await;
    let issues = out
        .get("issues")
        .and_then(|v| v.as_array())
        .expect("issues");
    let missing: Vec<_> = issues
        .iter()
        .filter(|i| i.get("type").and_then(|v| v.as_str()) == Some("missing_id"))
        .collect();
    assert_eq!(
        missing.len(),
        0,
        "expected no missing_id issues, got {issues:?}"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn validate_check_reports_status_and_nodes() {
    let ((_dir, _root, _engine, registry), _cwd) = setup_in_process().await;
    page_create(&registry, "tasks/validate-task", "Validate Task", "Body.").await;
    rebuild(&registry).await;
    let out = call_ok(&registry, "wm_validate.check", json!({})).await;
    assert!(out.get("status").is_some());
    assert!(out.get("nodes").is_some());
}

#[tokio::test(flavor = "multi_thread")]
async fn index_rebuild_and_status_agree() {
    let ((_dir, _root, _engine, registry), _cwd) = setup_in_process().await;
    page_create(&registry, "concepts/index-split", "Index Split", "Body.").await;
    let out = call_ok(&registry, "wm_index_rebuild", json!({ "skip_embed": true })).await;
    assert_eq!(out.get("status").and_then(|v| v.as_str()), Some("ok"));
    assert!(out.get("sections").is_some());

    let status = call_ok(&registry, "wm_index_status", json!({})).await;
    assert!(status.get("graph_nodes").is_some());
    assert!(status.get("sections").is_some());
}

/// The split index surface exposes three tools and drops the old `wm_index`.
#[tokio::test(flavor = "multi_thread")]
async fn index_split_tools_listed() {
    let ((_dir, _root, _engine, registry), _cwd) = setup_in_process().await;
    let tools = registry.list_tools();
    let names: Vec<&str> = tools
        .iter()
        .filter_map(|t| t.get("name").and_then(|n| n.as_str()))
        .collect();
    for name in ["wm_index_rebuild", "wm_index_status", "wm_index_embed"] {
        assert!(names.contains(&name), "missing split index tool: {name}");
    }
    assert!(
        !names.contains(&"wm_index"),
        "old wm_index tool must not exist"
    );
}

/// wm_index_embed must respond (success or an explicit model error) when forced.
#[tokio::test(flavor = "multi_thread")]
async fn index_embed_force_responds() {
    let ((_dir, _root, _engine, registry), _cwd) = setup_in_process().await;
    rebuild(&registry).await;
    match call(&registry, "wm_index_embed", json!({ "force": true })).await {
        Ok(res) => assert!(
            res.get("status").is_some(),
            "expected status in embed response, got {res}"
        ),
        Err(e) => assert!(
            e.message.contains("model")
                || e.message.contains("embed")
                || e.message.contains("sections"),
            "expected a model/embed error, got: {}",
            e.message
        ),
    }
}

// ---------------------------------------------------------------------------
// Tasks and workflow
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread")]
async fn task_update_todo_to_done_keeps_valid_frontmatter() {
    let ((_dir, root, _engine, registry), _cwd) = setup_in_process().await;
    let id = page_create(&registry, "tasks/todo-done-trans", "Todo to Done", "Body").await;
    call_ok(
        &registry,
        "wm_task",
        json!({ "action": "update", "id": id, "status": "done" }),
    )
    .await;
    let content = std::fs::read_to_string(root.join(".wm/wiki/tasks/todo-done-trans.md"))
        .expect("task file on disk");
    assert!(
        content.contains("status: done\n---"),
        "status must be followed by the closing delimiter, got:\n{content}"
    );
    assert!(
        !content.contains("done---"),
        "status must not glue to the delimiter"
    );
}

/// A spec/decision status must be rejected for task pages.
#[tokio::test(flavor = "multi_thread")]
async fn task_update_rejects_non_task_status() {
    let ((_dir, _root, _engine, registry), _cwd) = setup_in_process().await;
    let id = page_create(
        &registry,
        "tasks/task-status-bound",
        "Task Status Bound",
        "Body",
    )
    .await;
    let err = call_err(
        &registry,
        "wm_task",
        json!({ "action": "update", "id": id, "status": "approved" }),
    )
    .await;
    assert!(
        err.message
            .contains("Invalid status 'approved' for task page"),
        "expected a task-status validation error, got: {}",
        err.message
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn task_board_reflects_created_task() {
    let ((_dir, _root, _engine, registry), _cwd) = setup_in_process().await;
    page_create(&registry, "tasks/board-task", "Board Task", "Body").await;
    rebuild(&registry).await;
    let out = call_ok(&registry, "wm_task", json!({ "action": "board" })).await;
    let todo = out
        .get("counts")
        .and_then(|c| c.get("todo"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    assert!(todo >= 1, "expected the task on the todo column, got {out}");
}

#[tokio::test(flavor = "multi_thread")]
async fn task_lifecycle_time_tracking_and_retrieval() {
    let ((_dir, _root, _engine, registry), _cwd) = setup_in_process().await;
    let id = page_create(
        &registry,
        "tasks/lifecycle",
        "Lifecycle",
        "# Lifecycle\n\nTest task body.",
    )
    .await;
    rebuild(&registry).await;

    let listed = call_ok(&registry, "wm_page", json!({ "action": "list" })).await;
    let pages = listed
        .get("pages")
        .and_then(|v| v.as_array())
        .expect("pages");
    assert!(
        pages
            .iter()
            .any(|p| p.get("id").and_then(|v| v.as_str()) == Some(id.as_str())),
        "created page must appear in list"
    );

    let _ = call_ok(&registry, "wm_time", json!({ "action": "start", "id": id })).await;
    let _ = call_ok(&registry, "wm_time", json!({ "action": "stop", "id": id })).await;
    let report = call_ok(&registry, "wm_time", json!({ "action": "report" })).await;
    assert!(report.get("total_hours").is_some());

    let got = call_ok(&registry, "wm_page", json!({ "action": "get", "id": id })).await;
    assert_eq!(got.get("id").and_then(|v| v.as_str()), Some(id.as_str()));
}

/// Regression (#124 AC-3/4): wm_task.update preserves the full frontmatter
/// surface and never emits a `{}` block; get reflects updates immediately.
#[tokio::test(flavor = "multi_thread")]
async fn task_update_roundtrip_preserves_all_fields() {
    let ((_dir, root, _engine, registry), _cwd) = setup_in_process().await;
    let created = call_ok(
        &registry,
        "wm_task",
        json!({
            "action": "create",
            "title": "Roundtrip Task",
            "description": "Body desc.",
            "status": "todo",
            "priority": "high",
            "labels": ["alpha", "beta"],
            "acceptance_criteria": ["AC1", "AC2"],
        }),
    )
    .await;
    let id = created
        .get("id")
        .and_then(|v| v.as_str())
        .expect("task id")
        .to_string();

    let out = call_ok(
        &registry,
        "wm_task",
        json!({
            "action": "update",
            "id": id,
            "title": "Roundtrip Task V2",
            "status": "in-progress",
            "labels": ["alpha", "beta", "gamma"],
            "priority": "urgent",
            "implementation_plan": "plan here",
            "implementation_notes": "worked on it",
        }),
    )
    .await;
    assert_eq!(out.get("status").and_then(|v| v.as_str()), Some("updated"));

    let content = std::fs::read_to_string(root.join(".wm/wiki/tasks/roundtrip-task.md"))
        .expect("task file on disk");
    for needle in [
        "title: Roundtrip Task V2",
        "type: task",
        "id: \"wiki:tasks:roundtrip-task\"",
        "status: in-progress",
        "priority: urgent",
        "tags: [alpha, beta, gamma]",
        "acceptance_criteria:",
        "text: \"AC1\"",
        "implementation_plan: plan here",
        "implementation_notes: worked on it",
    ] {
        assert!(
            content.contains(needle),
            "missing {needle:?} in:\n{content}"
        );
    }
    assert!(
        !content.contains("{}"),
        "no '{{}}' block may be emitted, got:\n{content}"
    );

    let (fm, body) = wm_core::parser::extract_frontmatter(&content);
    let fm = fm.expect("frontmatter parses");
    assert_eq!(fm.title.as_deref(), Some("Roundtrip Task V2"));
    assert_eq!(fm.page_type.as_deref(), Some("task"));
    assert_eq!(fm.status.as_deref(), Some("in-progress"));
    assert_eq!(fm.priority.as_deref(), Some("urgent"));
    assert!(body.contains("Body desc."));

    let got = call_ok(&registry, "wm_task", json!({ "action": "get", "id": id })).await;
    assert_eq!(
        got.get("status").and_then(|v| v.as_str()),
        Some("in-progress")
    );
    assert_eq!(
        got.get("title").and_then(|v| v.as_str()),
        Some("Roundtrip Task V2")
    );
}

/// Regression (#124 AC-1/2): status transitions read fresh file state — get
/// reflects each transition immediately with no rebuild.
#[tokio::test(flavor = "multi_thread")]
async fn task_transition_get_is_fresh() {
    let ((_dir, root, _engine, registry), _cwd) = setup_in_process().await;
    let created = call_ok(
        &registry,
        "wm_task",
        json!({ "action": "create", "title": "Transition Task", "description": "Body.", "status": "todo" }),
    )
    .await;
    let id = created
        .get("id")
        .and_then(|v| v.as_str())
        .expect("task id")
        .to_string();

    call_ok(
        &registry,
        "wm_task",
        json!({ "action": "update", "id": id, "status": "in-progress" }),
    )
    .await;
    let got = call_ok(&registry, "wm_task", json!({ "action": "get", "id": id })).await;
    assert_eq!(
        got.get("status").and_then(|v| v.as_str()),
        Some("in-progress")
    );

    call_ok(
        &registry,
        "wm_task",
        json!({ "action": "update", "id": id, "status": "done" }),
    )
    .await;
    let got = call_ok(&registry, "wm_task", json!({ "action": "get", "id": id })).await;
    assert_eq!(got.get("status").and_then(|v| v.as_str()), Some("done"));

    let content = std::fs::read_to_string(root.join(".wm/wiki/tasks/transition-task.md"))
        .expect("task file on disk");
    assert!(
        content.contains("status: done\n"),
        "status must be valid yaml:\n{content}"
    );
    assert!(!content.contains("done---"));
    assert!(!content.contains("{}"));
}

/// Regression (#124): wm_page.link followed by wm_task.update must keep the
/// frontmatter intact (id/title/type/tags + relates_to) with no `{}` block.
#[tokio::test(flavor = "multi_thread")]
async fn task_link_then_update_preserves_frontmatter() {
    let ((_dir, root, _engine, registry), _cwd) = setup_in_process().await;
    let task_id = page_create(
        &registry,
        "tasks/link-update-task",
        "Link Update Task",
        "Body.",
    )
    .await;
    page_create(
        &registry,
        "specs/link-update-target",
        "Link Update Target",
        "Target.",
    )
    .await;

    call_ok(
        &registry,
        "wm_page",
        json!({
            "action": "link",
            "id": task_id,
            "target": "wiki:specs:link-update-target",
            "edge_type": "implements",
        }),
    )
    .await;

    call_ok(
        &registry,
        "wm_task",
        json!({
            "action": "update",
            "id": task_id,
            "title": "Link Update Task V2",
            "status": "in-progress",
            "labels": ["regression", "linked"],
        }),
    )
    .await;

    let content = std::fs::read_to_string(root.join(".wm/wiki/tasks/link-update-task.md"))
        .expect("task file on disk");
    for needle in [
        "title: Link Update Task V2",
        "type: task",
        "tags: [regression, linked]",
        "relates_to:",
        "implements",
        "wiki:specs:link-update-target",
    ] {
        assert!(
            content.contains(needle),
            "missing {needle:?} in:\n{content}"
        );
    }
    assert!(!content.contains("{}"), "no '{{}}' block, got:\n{content}");

    let got = call_ok(
        &registry,
        "wm_task",
        json!({ "action": "get", "id": task_id }),
    )
    .await;
    assert_eq!(
        got.get("status").and_then(|v| v.as_str()),
        Some("in-progress")
    );
}

/// Regression (#124 AC-2): a task in the in-memory graph reflects a status
/// update synchronously — get needs no rebuild and no watcher.
#[tokio::test(flavor = "multi_thread")]
async fn task_get_status_fresh_after_update() {
    let ((_dir, _root, _engine, registry), _cwd) = setup_in_process().await;
    let id = page_create(
        &registry,
        "tasks/indexed-status-task",
        "Indexed Status Task",
        "Body.",
    )
    .await;
    call_ok(
        &registry,
        "wm_task",
        json!({ "action": "update", "id": id, "status": "in-progress" }),
    )
    .await;
    let got = call_ok(&registry, "wm_task", json!({ "action": "get", "id": id })).await;
    assert_eq!(
        got.get("status").and_then(|v| v.as_str()),
        Some("in-progress"),
        "get must reflect the updated status immediately, got {got}"
    );
}

// ---------------------------------------------------------------------------
// Memory, template, source, decision, references
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread")]
async fn memory_add_creates_wiki_page() {
    let ((_dir, root, _engine, registry), _cwd) = setup_in_process().await;
    let out = call_ok(
        &registry,
        "wm_memory",
        json!({ "action": "add", "title": "Memory to Page", "content": "Test content", "tags": ["test"] }),
    )
    .await;
    let id = out.get("id").and_then(|v| v.as_str()).expect("memory id");
    let slug = id.rsplit(':').next().unwrap_or(id);
    let page_path = root.join(format!(".wm/wiki/memory/{slug}.md"));
    assert!(
        page_path.exists(),
        "memory page must be written at {} (id was {id})",
        page_path.display()
    );
}

/// Cross-entity search must surface both page and memory content.
#[tokio::test(flavor = "multi_thread")]
async fn cross_entity_search_finds_pages_and_memory() {
    let ((_dir, _root, _engine, registry), _cwd) = setup_in_process().await;
    page_create(
        &registry,
        "concepts/cross-entity",
        "Cross Entity Search Test",
        "Authentication tokens are verified via JWT.",
    )
    .await;
    rebuild(&registry).await;
    let _ = call_ok(
        &registry,
        "wm_memory",
        json!({
            "action": "add",
            "title": "Auth Pattern",
            "content": "Authentication uses JWT with RS256.",
            "tags": ["auth"],
        }),
    )
    .await;
    rebuild(&registry).await;

    let out = call_ok(
        &registry,
        "wm_search.query",
        json!({ "q": "authentication JWT", "type": "all", "limit": 20 }),
    )
    .await;
    let results = out
        .get("results")
        .and_then(|v| v.as_array())
        .expect("results");
    assert!(
        !results.is_empty(),
        "expected cross-entity results, got {out}"
    );
    assert!(
        results.iter().any(|r| r
            .get("id")
            .and_then(|v| v.as_str())
            .is_some_and(|id| id.contains("concepts:cross-entity"))),
        "expected a page result"
    );
    assert!(
        results.iter().any(|r| r
            .get("id")
            .and_then(|v| v.as_str())
            .is_some_and(|id| id.contains("memory:"))),
        "expected a memory result"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn template_create_then_list() {
    let ((_dir, _root, _engine, registry), _cwd) = setup_in_process().await;
    let initial = call_ok(&registry, "wm_template", json!({ "action": "list" })).await;
    assert_eq!(initial.get("total").and_then(|v| v.as_u64()), Some(0));

    let out = call_ok(
        &registry,
        "wm_template",
        json!({
            "action": "create",
            "name": "test-template",
            "description": "Test template",
            "content": "Hello {{name}}!",
        }),
    )
    .await;
    assert_eq!(out.get("status").and_then(|v| v.as_str()), Some("created"));

    let listed = call_ok(&registry, "wm_template", json!({ "action": "list" })).await;
    assert_eq!(listed.get("total").and_then(|v| v.as_u64()), Some(1));
    let templates = listed
        .get("templates")
        .and_then(|v| v.as_array())
        .expect("templates");
    assert!(
        templates
            .iter()
            .any(|t| t.get("name").and_then(|v| v.as_str()) == Some("test-template")),
        "created template must appear in list"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn source_list_reports_total() {
    let ((_dir, _root, _engine, registry), _cwd) = setup_in_process().await;
    let out = call_ok(&registry, "wm_source", json!({ "action": "list" })).await;
    assert!(out.get("sources").is_some());
    assert!(out.get("total").is_some());
}

#[tokio::test(flavor = "multi_thread")]
async fn decision_create_returns_identifier() {
    let ((_dir, _root, _engine, registry), _cwd) = setup_in_process().await;
    let out = call_ok(
        &registry,
        "wm_decision",
        json!({
            "action": "create",
            "id": "decisions/test-adr",
            "title": "Test ADR",
            "context": "We need to decide",
            "rationale": "Because",
            "outcome": "Option A",
        }),
    )
    .await;
    assert!(
        out.get("id").is_some() || out.get("status").is_some(),
        "expected a created decision, got {out}"
    );
}

/// wm_ref.extract must surface embedded @refs, including traversal-shaped ones
/// as opaque targets.
#[tokio::test(flavor = "multi_thread")]
async fn ref_extract_finds_references() {
    let ((_dir, _root, _engine, registry), _cwd) = setup_in_process().await;
    let out = call_ok(
        &registry,
        "wm_ref.extract",
        json!({ "content": "See @wiki/templates/../../etc/passwd for details." }),
    )
    .await;
    let refs = out
        .get("references")
        .and_then(|v| v.as_array())
        .expect("references");
    assert!(
        !refs.is_empty(),
        "expected at least one extracted reference, got {out}"
    );
}

// ---------------------------------------------------------------------------
// Schemas
// ---------------------------------------------------------------------------

/// Every tool must expose a flat object schema: no top-level oneOf/allOf/anyOf,
/// and action properties must carry a string enum.
#[tokio::test(flavor = "multi_thread")]
async fn all_tool_schemas_are_flat_objects() {
    let ((_dir, _root, _engine, registry), _cwd) = setup_in_process().await;
    let tools = registry.list_tools();
    assert!(tools.len() >= 30, "expected 30+ tools, got {}", tools.len());
    for tool in &tools {
        let name = tool.get("name").and_then(|v| v.as_str()).unwrap_or("?");
        let schema = tool
            .get("inputSchema")
            .unwrap_or_else(|| panic!("{name} missing inputSchema"));
        let obj = schema
            .as_object()
            .unwrap_or_else(|| panic!("{name} schema must be an object"));
        assert_eq!(
            obj.get("type").and_then(|v| v.as_str()),
            Some("object"),
            "{name} root type"
        );
        for keyword in ["oneOf", "allOf", "anyOf"] {
            assert!(!obj.contains_key(keyword), "{name} has top-level {keyword}");
        }
        if let Some(action) = obj
            .get("properties")
            .and_then(|v| v.get("action"))
            .and_then(|v| v.as_object())
        {
            let values = action
                .get("enum")
                .unwrap_or_else(|| panic!("{name} action needs an enum"));
            assert!(
                values
                    .as_array()
                    .is_some_and(|arr| arr.iter().any(|v| v.is_string())),
                "{name} action enum values must be strings"
            );
        }
    }
}

/// wm_page's schema must require only `action`, list its enum, carry
/// descriptions on every non-action field, and never expose `page_id`.
#[tokio::test(flavor = "multi_thread")]
async fn wm_page_schema_contract() {
    let ((_dir, _root, _engine, registry), _cwd) = setup_in_process().await;
    let tools = registry.list_tools();
    let page_tool = tools
        .iter()
        .find(|t| t.get("name").and_then(|v| v.as_str()) == Some("wm_page"))
        .expect("wm_page listed");
    let schema = page_tool
        .get("inputSchema")
        .expect("inputSchema")
        .as_object()
        .expect("object");
    assert!(schema.get("oneOf").is_none());
    let required = schema
        .get("required")
        .and_then(|v| v.as_array())
        .expect("required");
    assert_eq!(required.len(), 1, "wm_page must require only action");
    assert!(required.iter().any(|r| r.as_str() == Some("action")));

    let props = schema
        .get("properties")
        .and_then(|v| v.as_object())
        .expect("properties");
    let action = props
        .get("action")
        .and_then(|v| v.as_object())
        .expect("action property");
    let values = action
        .get("enum")
        .and_then(|v| v.as_array())
        .expect("action enum");
    assert_eq!(values.len(), 7, "expected 7 wm_page actions");
    for expected in [
        "list", "get", "create", "update", "delete", "link", "unlink",
    ] {
        assert!(
            values.iter().any(|v| v.as_str() == Some(expected)),
            "missing {expected}"
        );
    }
    for (name, prop) in props {
        assert!(
            name == "action" || prop.get("description").is_some(),
            "wm_page.{name} missing description"
        );
        assert_ne!(
            name, "page_id",
            "wm_page must not expose a page_id parameter"
        );
    }
}

// ---------------------------------------------------------------------------
// Unknown tool dispatch
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread")]
async fn unknown_tool_is_rejected() {
    let ((_dir, _root, _engine, registry), _cwd) = setup_in_process().await;
    let err = call_err(&registry, "wm_nonexistent", json!({})).await;
    assert!(
        err.message.contains("Unknown") || err.message.contains("not found"),
        "expected an unknown-tool error, got: {}",
        err.message
    );
}

// ---------------------------------------------------------------------------
// Code intelligence
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread")]
async fn code_search_finds_and_does_not_find() {
    let ((_dir, root, _engine, registry), _cwd) = setup_in_process().await;
    std::fs::create_dir_all(root.join("src")).expect("create src");
    std::fs::write(
        root.join("src/wm_lib.rs"),
        "pub struct CodeTest {}\npub fn greet() {}\n",
    )
    .expect("write source");

    let found = call_ok(
        &registry,
        "wm_code.search",
        json!({ "pattern": "pub struct", "max_results": 10 }),
    )
    .await;
    let results = found
        .get("results")
        .and_then(|v| v.as_array())
        .expect("results");
    assert!(!results.is_empty(), "should find 'pub struct'");
    assert!(found.get("total").and_then(|v| v.as_u64()).unwrap_or(0) >= 1);

    let none = call_ok(
        &registry,
        "wm_code.search",
        json!({ "pattern": "ZZZZNOTFOUND" }),
    )
    .await;
    assert_eq!(none.get("total").and_then(|v| v.as_u64()), Some(0));

    let invalid = call(
        &registry,
        "wm_code.search",
        json!({ "pattern": "[invalid" }),
    )
    .await;
    assert!(invalid.is_err(), "invalid regex must error");
}

#[tokio::test(flavor = "multi_thread")]
async fn code_symbols_index_kinds() {
    let ((_dir, root, _engine, registry), _cwd) = setup_in_process().await;
    std::fs::create_dir_all(root.join("src")).expect("create src");
    std::fs::write(
        root.join("src/wm_lib.rs"),
        "pub struct CodeTest {}\npub fn greet() {}\npub enum Status { Active }\npub trait Processor {}\n",
    )
    .expect("write source");

    for (kind, expected) in [
        ("struct", "CodeTest"),
        ("function", "greet"),
        ("enum", "Status"),
        ("trait", "Processor"),
    ] {
        let out = call_ok(&registry, "wm_code.symbols", json!({ "kind": kind })).await;
        let symbols = out
            .get("symbols")
            .and_then(|v| v.as_array())
            .expect("symbols");
        assert!(
            symbols
                .iter()
                .any(|s| s.get("name").and_then(|n| n.as_str()) == Some(expected)),
            "kind {kind} should include {expected}, got {symbols:?}"
        );
    }

    let by_name = call_ok(&registry, "wm_code.symbols", json!({ "name": "CodeTest" })).await;
    let symbols = by_name
        .get("symbols")
        .and_then(|v| v.as_array())
        .expect("symbols");
    assert!(!symbols.is_empty(), "name filter should match CodeTest");
    for sym in symbols {
        assert!(sym
            .get("name")
            .and_then(|n| n.as_str())
            .unwrap_or("")
            .contains("CodeTest"));
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn code_deps_and_tool_surface() {
    let ((_dir, root, _engine, registry), _cwd) = setup_in_process().await;
    std::fs::create_dir_all(root.join("src")).expect("create src");
    std::fs::write(
        root.join("src/wm_main.rs"),
        "mod lib;\nuse lib::CodeTest;\nfn main() {}\n",
    )
    .expect("write main");
    std::fs::write(root.join("src/wm_lib.rs"), "pub struct CodeTest {}\n").expect("write lib");

    let out = call_ok(&registry, "wm_code.deps", json!({})).await;
    let deps = out
        .get("dependencies")
        .and_then(|v| v.as_array())
        .expect("dependencies");
    assert!(
        deps.iter().any(|d| d
            .get("file")
            .and_then(|f| f.as_str())
            .is_some_and(|f| f.contains("wm_main.rs"))),
        "main.rs should have dependencies, got {deps:?}"
    );

    let tools = registry.list_tools();
    let names: Vec<&str> = tools
        .iter()
        .filter_map(|t| t.get("name").and_then(|n| n.as_str()))
        .collect();
    for tool in ["wm_code.search", "wm_code.symbols", "wm_code.deps"] {
        assert!(names.contains(&tool), "missing {tool}");
    }
}

// ---------------------------------------------------------------------------
// Frontmatter builder choke point
// ---------------------------------------------------------------------------

/// A task (and subtask) whose title starts with '[' and contains ':' must
/// never corrupt the file: the frontmatter must parse back and the task must
/// stay resolvable via wm_task.get. Guards the single frontmatter-builder
/// choke point that every string-built CREATE path routes through.
#[tokio::test(flavor = "multi_thread")]
async fn task_create_with_yaml_breaking_title_round_trips() {
    let ((_dir, root, _engine, registry), _cwd) = setup_in_process().await;

    let nasty_title = "[BLOCK]: fix: the thing";

    let created = call_ok(
        &registry,
        "wm_task",
        json!({ "action": "create", "title": nasty_title, "description": "Body." }),
    )
    .await;
    let task_id = created
        .get("id")
        .and_then(|v| v.as_str())
        .expect("task id")
        .to_string();

    let task_path = root.join(".wm/wiki").join(format!(
        "{}.md",
        task_id.trim_start_matches("wiki:").replace(':', "/")
    ));
    let content = std::fs::read_to_string(&task_path).expect("task file on disk");
    let (fm, _body) = wm_core::parser::extract_frontmatter(&content);
    let fm = fm.expect("task frontmatter must parse back");
    assert_eq!(
        fm.title.as_deref(),
        Some(nasty_title),
        "title must round-trip, got:\n{content}"
    );

    let got = call_ok(
        &registry,
        "wm_task",
        json!({ "action": "get", "id": task_id }),
    )
    .await;
    assert_eq!(got.get("title").and_then(|v| v.as_str()), Some(nasty_title));

    let sub_created = call_ok(
        &registry,
        "wm_task",
        json!({ "action": "subtask", "id": task_id, "title": nasty_title }),
    )
    .await;
    let sub_id = sub_created
        .get("id")
        .and_then(|v| v.as_str())
        .expect("subtask id")
        .to_string();

    let sub_path = root.join(".wm/wiki").join(format!(
        "{}.md",
        sub_id.trim_start_matches("wiki:").replace(':', "/")
    ));
    let sub_content = std::fs::read_to_string(&sub_path).expect("subtask file on disk");
    let (sub_fm, _sub_body) = wm_core::parser::extract_frontmatter(&sub_content);
    let sub_fm = sub_fm.expect("subtask frontmatter must parse back");
    assert_eq!(
        sub_fm.title.as_deref(),
        Some(nasty_title),
        "subtask title must round-trip, got:\n{sub_content}"
    );

    let sub_got = call_ok(
        &registry,
        "wm_task",
        json!({ "action": "get", "id": sub_id }),
    )
    .await;
    assert_eq!(
        sub_got.get("title").and_then(|v| v.as_str()),
        Some(nasty_title)
    );
}
