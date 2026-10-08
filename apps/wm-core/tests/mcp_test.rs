
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

    let err = client
        .call_tool(
            "wm_page",
            json!({ "action": "get", "id": "nonexistent:id" }),
        )
        .expect_err("unknown page must error through the transport");
    assert!(err.contains("not found"), "got: {err}");
}

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

fn format_warning(out: &serde_json::Value) -> Option<String> {
    out.get("format_warning")
        .and_then(|v| v.as_str())
        .map(str::to_owned)
}

#[tokio::test(flavor = "multi_thread")]
async fn page_get_flags_prose_body_on_record_bearing_type() {
    let ((_dir, _root, _engine, registry), _cwd) = setup_in_process().await;
    page_create(
        &registry,
        "concepts/prose-record",
        "Prose Record",
        "## Context\n\nThis is prose, not a record.\n",
    )
    .await;
    let out = call_ok(
        &registry,
        "wm_page",
        json!({ "action": "get", "id": "wiki:concepts:prose-record" }),
    )
    .await;
    let warning = format_warning(&out).expect("prose record-bearing page must warn");
    assert!(
        warning.contains("migrate-records"),
        "hint must name the migration command, got: {warning}"
    );
    assert!(
        warning.contains("typed-decision-record-schema"),
        "hint must link the schema, got: {warning}"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn page_get_flags_empty_body_on_record_bearing_type() {
    let ((_dir, root, _engine, registry), _cwd) = setup_in_process().await;
    std::fs::create_dir_all(root.join(".wm/wiki/decisions")).expect("create decisions dir");
    std::fs::write(
        root.join(".wm/wiki/decisions/wm-self-upgrade.md"),
        "---\ntitle: Self Upgrade\ntype: decision\nid: \"wiki:decisions:wm-self-upgrade\"\n---\n",
    )
    .expect("write empty-body decision");
    let out = call_ok(
        &registry,
        "wm_page",
        json!({ "action": "get", "id": "wiki:decisions:wm-self-upgrade" }),
    )
    .await;
    assert!(
        format_warning(&out).is_some(),
        "empty-body record-bearing page must warn, got: {out}"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn page_get_does_not_flag_valid_record() {
    let ((_dir, _root, _engine, registry), _cwd) = setup_in_process().await;
    let body = "schema_version: 1\nstate: |-\n  ## Context\n\n  Chose SQLite.\nquestions:\n  - id: outcome\n    type: choice\n    instructions: What is the recorded outcome of this decision?\n    options: [adopted, rejected, deferred, superseded, abandoned]\n  - id: reversibility\n    type: noul\n    instructions: The decision can be reversed cheaply without data migration or cross-module breakage.\n  - id: confidence\n    type: score\n    instructions: How strong is the recorded justification for the selected outcome?\n    levels: [low, medium, high]\n  - id: impact\n    type: choice\n    instructions: How wide is the blast radius of this decision?\n    options: [local, component, system, project-wide]\nanswers: {}\n";
    page_create(&registry, "decisions/valid-record", "Valid Record", body).await;
    let out = call_ok(
        &registry,
        "wm_page",
        json!({ "action": "get", "id": "wiki:decisions:valid-record" }),
    )
    .await;
    assert!(
        format_warning(&out).is_none(),
        "valid record must not warn, got: {out}"
    );
}

const DECISION_STATE_BODY: &str = "schema_version: 1\nstate: |-\n  ## Context\n\n  Chose SQLite over Postgres.\nquestions:\n  - id: outcome\n    type: choice\n    instructions: What is the recorded outcome of this decision?\n    options: [adopted, rejected]\nanswers: {}\n";

const MEMORY_RECORD_BODY: &str = "schema_version: 1\nstate: |-\n  ## Memory\n\n  Prefer line-based YAML edits; token uniformstructtoken.\nquestions:\n  - id: layer\n    type: choice\n    instructions: Which memory layer does this entry belong to?\n    options: [project, global, session]\n  - id: store_or_skip\n    type: noul\n    instructions: This entry is worth storing as durable memory.\n  - id: dedup_action\n    type: choice\n    instructions: How should this entry relate to existing memory?\n    options: [new, merge, supersede, skip]\n  - id: confidence\n    type: score\n    instructions: How confident is the recorded knowledge?\n    levels: [low, medium, high]\nanswers: {}\n";

#[tokio::test(flavor = "multi_thread")]
async fn page_get_returns_state_as_content_for_records() {
    let ((_dir, _root, _engine, registry), _cwd) = setup_in_process().await;
    page_create(
        &registry,
        "decisions/state-content",
        "State Content",
        DECISION_STATE_BODY,
    )
    .await;
    let out = call_ok(
        &registry,
        "wm_page",
        json!({ "action": "get", "id": "wiki:decisions:state-content" }),
    )
    .await;
    let content = out.get("content").and_then(|v| v.as_str()).unwrap_or("");
    assert_eq!(content, "## Context\n\nChose SQLite over Postgres.");
    assert!(!content.contains("schema_version"), "raw YAML must not leak");
    assert!(out.get("record").is_some(), "parsed record must be present");
}

const RULE_RECORD_BODY: &str = "schema_version: 1\nstate: |-\n  ## Rule\n\n  Always run cargo test before commit.\nquestions:\n  - id: enforcement\n    type: choice\n    instructions: How is this rule enforced?\n    options: [ci-enforced, tool-enforced, review-enforced, convention-only]\n  - id: severity\n    type: score\n    instructions: How severe is a violation of this rule?\n    levels: [advisory, recommended, required, blocking]\n  - id: has_exception\n    type: noul\n    instructions: This rule has documented exceptions.\n  - id: applies_to\n    type: choice\n    multi: true\n    instructions: Which surfaces does this rule apply to?\n    options: [code, tests, docs, configuration, workflow, security]\nanswers: {}\n";

#[tokio::test(flavor = "multi_thread")]
async fn page_get_returns_state_for_rule() {
    let ((_dir, _root, _engine, registry), _cwd) = setup_in_process().await;
    page_create(&registry, "rules/always-test", "Always Test", RULE_RECORD_BODY).await;
    let out = call_ok(
        &registry,
        "wm_page",
        json!({ "action": "get", "id": "wiki:rules:always-test" }),
    )
    .await;
    let content = out.get("content").and_then(|v| v.as_str()).unwrap_or("");
    assert!(content.contains("Always run cargo test before commit."), "got {out}");
    assert!(!content.contains("schema_version"), "raw YAML must not leak");
}

#[tokio::test(flavor = "multi_thread")]
async fn memory_get_returns_state_for_record_pages() {
    let ((_dir, _root, _engine, registry), _cwd) = setup_in_process().await;
    page_create(
        &registry,
        "memory/record-mem",
        "Record Mem",
        MEMORY_RECORD_BODY,
    )
    .await;
    let out = call_ok(
        &registry,
        "wm_memory",
        json!({ "action": "get", "id": "wiki:memory:record-mem" }),
    )
    .await;
    let content = out.get("content").and_then(|v| v.as_str()).unwrap_or("");
    assert!(content.contains("line-based YAML edits"), "got {out}");
    assert!(!content.contains("schema_version"), "raw YAML must not leak");
}

#[tokio::test(flavor = "multi_thread")]
async fn search_retrieve_injects_memory_state() {
    let ((_dir, _root, _engine, registry), _cwd) = setup_in_process().await;
    page_create(
        &registry,
        "memory/search-mem",
        "Search Mem",
        MEMORY_RECORD_BODY,
    )
    .await;
    rebuild(&registry).await;
    let out = call_ok(
        &registry,
        "wm_search.retrieve",
        json!({ "q": "uniformstructtoken", "type": "memory", "token_budget": 8000 }),
    )
    .await;
    let context = out.get("context").and_then(|v| v.as_str()).unwrap_or("");
    assert!(
        context.contains("line-based YAML edits"),
        "memory context must carry state, got {out}"
    );
    assert!(
        !context.contains("schema_version"),
        "raw YAML must not leak into context"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn page_get_flags_prose_rule_now_record_bearing() {
    let ((_dir, _root, _engine, registry), _cwd) = setup_in_process().await;
    page_create(
        &registry,
        "rules/prose-rule",
        "Prose Rule",
        "## Rule\n\nDo the thing.\n",
    )
    .await;
    let out = call_ok(
        &registry,
        "wm_page",
        json!({ "action": "get", "id": "wiki:rules:prose-rule" }),
    )
    .await;
    assert!(
        format_warning(&out).is_some(),
        "prose rule must warn once rule is record-bearing, got: {out}"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn page_update_flags_prose_body_on_record_bearing_type() {
    let ((_dir, _root, _engine, registry), _cwd) = setup_in_process().await;
    page_create(
        &registry,
        "concepts/update-prose",
        "Update Prose",
        "## Context\n\nOriginal prose.\n",
    )
    .await;
    let out = call_ok(
        &registry,
        "wm_page",
        json!({
            "action": "update",
            "id": "wiki:concepts:update-prose",
            "content": "## Context\n\nStill prose, not a record.\n",
        }),
    )
    .await;
    assert_eq!(out.get("status").and_then(|v| v.as_str()), Some("updated"));
    assert!(
        format_warning(&out).is_some(),
        "update with prose body must warn, got: {out}"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn page_update_does_not_flag_valid_record() {
    let ((_dir, _root, _engine, registry), _cwd) = setup_in_process().await;
    let body = "schema_version: 1\nstate: |-\n  ## Context\n\n  Chose SQLite.\nquestions:\n  - id: outcome\n    type: choice\n    instructions: What is the recorded outcome of this decision?\n    options: [adopted, rejected, deferred, superseded, abandoned]\n  - id: reversibility\n    type: noul\n    instructions: The decision can be reversed cheaply without data migration or cross-module breakage.\n  - id: confidence\n    type: score\n    instructions: How strong is the recorded justification for the selected outcome?\n    levels: [low, medium, high]\n  - id: impact\n    type: choice\n    instructions: How wide is the blast radius of this decision?\n    options: [local, component, system, project-wide]\nanswers: {}\n";
    page_create(&registry, "decisions/update-valid", "Update Valid", body).await;
    let out = call_ok(
        &registry,
        "wm_page",
        json!({
            "action": "update",
            "id": "wiki:decisions:update-valid",
            "content": body,
        }),
    )
    .await;
    assert!(
        format_warning(&out).is_none(),
        "valid record update must not warn, got: {out}"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn doc_get_flags_prose_body_on_record_bearing_type() {
    let ((_dir, root, _engine, registry), _cwd) = setup_in_process().await;
    std::fs::create_dir_all(root.join(".wm/wiki/howto")).expect("create howto dir");
    std::fs::write(
        root.join(".wm/wiki/howto/legacy-prose.md"),
        "---\ntitle: Legacy Prose\ntype: howto\n---\n\nJust prose.\n",
    )
    .expect("write legacy prose howto");
    let out = call_ok(
        &registry,
        "wm_doc",
        json!({ "action": "get", "path": "howto/legacy-prose" }),
    )
    .await;
    assert!(
        format_warning(&out).is_some(),
        "wm_doc.get must warn on old-format record-bearing page, got: {out}"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn doc_update_flags_prose_body_on_record_bearing_type() {
    let ((_dir, _root, _engine, registry), _cwd) = setup_in_process().await;
    call_ok(
        &registry,
        "wm_doc",
        json!({
            "action": "create",
            "path": "concepts/doc-prose",
            "title": "Doc Prose",
            "type": "concept",
            "content": "## Context\n\nProse.\n",
        }),
    )
    .await;
    let out = call_ok(
        &registry,
        "wm_doc",
        json!({
            "action": "update",
            "path": "concepts/doc-prose",
            "content": "## Context\n\nStill prose.\n",
        }),
    )
    .await;
    assert!(
        format_warning(&out).is_some(),
        "wm_doc.update must warn on prose record-bearing page, got: {out}"
    );
}


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

fn record_errors(out: &serde_json::Value) -> Vec<serde_json::Value> {
    out.get("errors")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter(|error| {
            error
                .get("field")
                .and_then(|v| v.as_str())
                .map(|field| field.starts_with("record"))
                .unwrap_or(false)
        })
        .collect()
}

fn record_error_fields(out: &serde_json::Value) -> Vec<String> {
    record_errors(out)
        .iter()
        .filter_map(|error| error.get("field").and_then(|v| v.as_str()))
        .map(str::to_owned)
        .collect()
}

#[tokio::test(flavor = "multi_thread")]
async fn search_finds_page_written_outside_watcher() {
    let ((_dir, root, _engine, registry), _cwd) = setup_in_process().await;
    std::fs::create_dir_all(root.join(".wm/wiki/concepts")).expect("create concepts dir");
    std::fs::write(
        root.join(".wm/wiki/concepts/external-fresh.md"),
        "---\ntitle: External Fresh\ntype: concept\n---\n\n## Body\n\nUnique zzexternalzz phrase.\n",
    )
    .expect("write external page");
    let out = call_ok(
        &registry,
        "wm_search.query",
        json!({ "q": "zzexternalzz", "type": "all", "limit": 10 }),
    )
    .await;
    let results = out
        .get("results")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    assert!(
        results.iter().any(|result| result
            .get("id")
            .and_then(|v| v.as_str())
            .is_some_and(|id| id.contains("external-fresh"))),
        "page written outside the watcher must be fresh for search: {out}"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn validate_refreshes_page_written_outside_watcher() {
    let ((_dir, root, _engine, registry), _cwd) = setup_in_process().await;
    std::fs::create_dir_all(root.join(".wm/wiki/rules")).expect("create rules dir");
    std::fs::write(
        root.join(".wm/wiki/rules/external-rule.md"),
        "---\ntitle: External Rule\ntype: rule\n---\n\nProse rule.\n",
    )
    .expect("write external rule");
    let out = call_ok(&registry, "wm_validate.check", json!({})).await;
    assert!(
        record_errors(&out).iter().any(|error| error
            .get("id")
            .and_then(|v| v.as_str())
            .is_some_and(|id| id == "wiki:rules:external-rule")),
        "external page must be validated after refresh: {out}"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn search_finds_spec_frontmatter_requirement() {
    let ((_dir, root, _engine, registry), _cwd) = setup_in_process().await;
    std::fs::create_dir_all(root.join(".wm/wiki/specs")).expect("create specs dir");
    std::fs::write(
        root.join(".wm/wiki/specs/fr-indexing.md"),
        "---\ntitle: FR Indexing Spec\ntype: spec\nfunctional_requirements:\n  - {id: FR-9, description: \"Requirement phrase zzfrphrasezz for keyword search\"}\n---\n\n## Overview\n\nSpec body prose.\n",
    )
    .expect("write spec");
    let out = call_ok(
        &registry,
        "wm_search.query",
        json!({ "q": "zzfrphrasezz", "type": "all", "limit": 10 }),
    )
    .await;
    let results = out
        .get("results")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    assert!(
        results.iter().any(|result| result
            .get("id")
            .and_then(|v| v.as_str())
            .is_some_and(|id| id.contains("fr-indexing"))),
        "spec frontmatter requirement must be findable: {out}"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn index_status_reports_actionable_degraded_reason_without_model() {
    let ((_dir, _root, engine, registry), _cwd) = setup_in_process().await;
    if engine.embedder.is_loaded() {
        return;
    }
    let out = call_ok(&registry, "wm_index_status", json!({})).await;
    assert_eq!(out["degraded"], true);
    let reason = out["degraded_reason"].as_str().unwrap_or("");
    assert!(reason.contains("wm model download"), "{out}");
}

#[tokio::test(flavor = "multi_thread")]
async fn search_reports_actionable_degraded_reason_without_model() {
    let ((_dir, _root, engine, registry), _cwd) = setup_in_process().await;
    if engine.embedder.is_loaded() {
        return;
    }
    let out = call_ok(
        &registry,
        "wm_search.query",
        json!({ "q": "anything", "type": "all" }),
    )
    .await;
    assert_eq!(out["degraded"], true);
    assert!(
        out["warning"]
            .as_str()
            .unwrap_or("")
            .contains("wm model download"),
        "{out}"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn retrieve_reports_actionable_degraded_reason_without_model() {
    let ((_dir, _root, engine, registry), _cwd) = setup_in_process().await;
    if engine.embedder.is_loaded() {
        return;
    }
    let out = call_ok(
        &registry,
        "wm_search.retrieve",
        json!({ "q": "anything", "token_budget": 2000 }),
    )
    .await;
    assert_eq!(out["degraded"], true);
    assert!(
        out["warning"]
            .as_str()
            .unwrap_or("")
            .contains("wm model download"),
        "{out}"
    );
}

async fn validate_record_body(registry: &ToolRegistry, path: &str, body: &str) -> serde_json::Value {
    page_create(registry, path, "Record Page", body).await;
    rebuild(registry).await;
    call_ok(registry, "wm_validate.check", json!({})).await
}

#[tokio::test(flavor = "multi_thread")]
async fn validate_accepts_a_valid_converted_record() {
    let ((_dir, _root, _engine, registry), _cwd) = setup_in_process().await;
    let body = "schema_version: 1\nstate: |-\n  ## Context\n\n  Chose SQLite.\nquestions:\n  - id: outcome\n    type: choice\n    instructions: What is the recorded outcome of this decision?\n    options: [adopted, rejected, deferred, superseded, abandoned]\n  - id: reversibility\n    type: noul\n    instructions: The decision can be reversed cheaply without data migration or cross-module breakage.\n  - id: confidence\n    type: score\n    instructions: How strong is the recorded justification for the selected outcome?\n    levels: [low, medium, high]\n  - id: impact\n    type: choice\n    instructions: How wide is the blast radius of this decision?\n    options: [local, component, system, project-wide]\nanswers: {}\n";
    let out = validate_record_body(&registry, "decisions/valid-record", body).await;
    assert!(
        record_errors(&out).is_empty(),
        "valid record must not produce record errors: {:?}",
        record_errors(&out)
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn validate_reports_missing_state() {
    let ((_dir, _root, _engine, registry), _cwd) = setup_in_process().await;
    let body = "schema_version: 1\nquestions: []\n";
    let out = validate_record_body(&registry, "concepts/missing-state", body).await;
    assert!(
        record_error_fields(&out).contains(&"record.state".to_owned()),
        "expected record.state error, got {:?}",
        record_errors(&out)
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn validate_reports_bad_question_id() {
    let ((_dir, _root, _engine, registry), _cwd) = setup_in_process().await;
    let body = "schema_version: 1\nstate: |-\n  prose\nquestions:\n  - id: Bad\n    type: choice\n    instructions: Pick one.\n    options: [a, b]\nanswers: {}\n";
    let out = validate_record_body(&registry, "concepts/bad-question-id", body).await;
    assert!(
        record_error_fields(&out).contains(&"record.questions[Bad].id".to_owned()),
        "expected question id error, got {:?}",
        record_errors(&out)
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn validate_reports_bad_question_type() {
    let ((_dir, _root, _engine, registry), _cwd) = setup_in_process().await;
    let body = "schema_version: 1\nstate: |-\n  prose\nquestions:\n  - id: outcome\n    type: bogus\n    instructions: Pick one.\n    options: [a, b]\nanswers: {}\n";
    let out = validate_record_body(&registry, "concepts/bad-question-type", body).await;
    assert!(
        record_error_fields(&out).contains(&"record.questions[outcome].type".to_owned()),
        "expected question type error, got {:?}",
        record_errors(&out)
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn validate_reports_bad_instructions() {
    let ((_dir, _root, _engine, registry), _cwd) = setup_in_process().await;
    let body = "schema_version: 1\nstate: |-\n  prose\nquestions:\n  - id: outcome\n    type: choice\n    instructions: \"\"\n    options: [a, b]\nanswers: {}\n";
    let out = validate_record_body(&registry, "concepts/bad-instructions", body).await;
    assert!(
        record_error_fields(&out).contains(&"record.questions[outcome].instructions".to_owned()),
        "expected instructions error, got {:?}",
        record_errors(&out)
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn validate_reports_choice_option_count() {
    let ((_dir, _root, _engine, registry), _cwd) = setup_in_process().await;
    let body = "schema_version: 1\nstate: |-\n  prose\nquestions:\n  - id: outcome\n    type: choice\n    instructions: Pick one.\n    options: [only]\nanswers: {}\n";
    let out = validate_record_body(&registry, "concepts/option-count", body).await;
    assert!(
        record_error_fields(&out).contains(&"record.questions[outcome].options".to_owned()),
        "expected options error, got {:?}",
        record_errors(&out)
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn validate_reports_score_level_count() {
    let ((_dir, _root, _engine, registry), _cwd) = setup_in_process().await;
    let body = "schema_version: 1\nstate: |-\n  prose\nquestions:\n  - id: confidence\n    type: score\n    instructions: How strong?\n    levels: [only]\nanswers: {}\n";
    let out = validate_record_body(&registry, "concepts/level-count", body).await;
    assert!(
        record_error_fields(&out).contains(&"record.questions[confidence].levels".to_owned()),
        "expected levels error, got {:?}",
        record_errors(&out)
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn validate_reports_multi_on_non_choice() {
    let ((_dir, _root, _engine, registry), _cwd) = setup_in_process().await;
    let body = "schema_version: 1\nstate: |-\n  prose\nquestions:\n  - id: confidence\n    type: score\n    multi: true\n    instructions: How strong?\n    levels: [low, high]\nanswers: {}\n";
    let out = validate_record_body(&registry, "concepts/multi-non-choice", body).await;
    assert!(
        record_error_fields(&out).contains(&"record.questions[confidence].multi".to_owned()),
        "expected multi error, got {:?}",
        record_errors(&out)
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn validate_reports_unknown_answer_key() {
    let ((_dir, _root, _engine, registry), _cwd) = setup_in_process().await;
    let body = "schema_version: 1\nstate: |-\n  prose\nquestions:\n  - id: outcome\n    type: choice\n    instructions: Pick one.\n    options: [a, b]\nanswers:\n  mystery: a\n";
    let out = validate_record_body(&registry, "concepts/unknown-answer", body).await;
    assert!(
        record_error_fields(&out).contains(&"record.answers.mystery".to_owned()),
        "expected unknown answer error, got {:?}",
        record_errors(&out)
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn validate_reports_invalid_answer_value() {
    let ((_dir, _root, _engine, registry), _cwd) = setup_in_process().await;
    let body = "schema_version: 1\nstate: |-\n  prose\nquestions:\n  - id: outcome\n    type: choice\n    instructions: Pick one.\n    options: [a, b]\nanswers:\n  outcome: nonsense\n";
    let out = validate_record_body(&registry, "concepts/invalid-answer", body).await;
    assert!(
        record_error_fields(&out).contains(&"record.answers.outcome".to_owned()),
        "expected invalid answer error, got {:?}",
        record_errors(&out)
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn validate_reports_canonical_mismatch() {
    let ((_dir, _root, _engine, registry), _cwd) = setup_in_process().await;
    let body = "schema_version: 1\nstate: |-\n  prose\nquestions:\n  - id: outcome\n    type: choice\n    instructions: Pick one.\n    options: [a, b]\nanswers: {}\n";
    let out = validate_record_body(&registry, "decisions/canonical-mismatch", body).await;
    assert!(
        record_error_fields(&out).contains(&"record.questions".to_owned()),
        "expected canonical mismatch error, got {:?}",
        record_errors(&out)
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn validate_flags_prose_on_all_page_types() {
    let ((_dir, _root, _engine, registry), _cwd) = setup_in_process().await;
    page_create(
        &registry,
        "rules/prose-rule",
        "Prose Rule",
        "## Rule\n\nDo the thing.\n",
    )
    .await;
    rebuild(&registry).await;
    let out = call_ok(&registry, "wm_validate.check", json!({})).await;
    let errors = record_errors(&out);
    assert!(
        errors.iter().any(|error| {
            error
                .get("id")
                .and_then(|v| v.as_str())
                .is_some_and(|id| id.contains("rules:prose-rule"))
                && error.get("field").and_then(|v| v.as_str()) == Some("record")
        }),
        "prose rule must produce a record error: {errors:?}"
    );
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

#[tokio::test(flavor = "multi_thread")]
async fn decision_create_writes_parseable_record_body() {
    let ((_dir, root, _engine, registry), _cwd) = setup_in_process().await;
    call_ok(
        &registry,
        "wm_decision",
        json!({
            "action": "create",
            "id": "decisions/record-body-adr",
            "title": "Record Body ADR",
            "context": "We need to decide",
            "rationale": "Because",
            "outcome": "Option A",
        }),
    )
    .await;

    let content = std::fs::read_to_string(root.join(".wm/wiki/decisions/record-body-adr.md"))
        .expect("created decision page must exist");
    let (_frontmatter, body) = wm_core::parser::extract_frontmatter(&content);
    let record = wm_engine::parse_record(&wm_engine::PageType::Decision, body)
        .expect("created decision body must be a parseable record");
    assert_eq!(record.schema_version, wm_engine::RECORD_SCHEMA_VERSION);
    assert!(record.state.contains("We need to decide"));
    assert_eq!(
        record.questions,
        wm_engine::canonical_questions(&wm_engine::PageType::Decision)
    );
    assert!(record.answers.is_empty());
}

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
