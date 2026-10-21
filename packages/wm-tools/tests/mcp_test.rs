#[path = "helpers/mcp.rs"]
mod helpers;
use helpers::MCPClient;

#[path = "helpers/inproc.rs"]
mod inproc;
use inproc::{call, call_err, call_ok, setup_in_process};

use serde_json::json;
use wm_tools::transport::ToolRegistry;

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
    assert_eq!(
        tools.len(),
        registry.list_tools().len(),
        "wm_help cache must match the registry tool list"
    );
    for name in ["wm_initial", "wm_help", "wm_page", "wm_search.query"] {
        assert!(
            tools
                .iter()
                .any(|tool| tool.get("name").and_then(|value| value.as_str()) == Some(name)),
            "missing cached tool: {name}"
        );
    }
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
    let (fm, _body) = wm_engine::parser::extract_frontmatter(&content);
    let fm = fm.expect("frontmatter parses");
    assert_eq!(fm.page_type.as_str(), "pattern");
    assert_eq!(fm.title, "Extra FM");
}

#[tokio::test(flavor = "multi_thread")]
async fn page_update_concurrent_writes_do_not_return_io_error() {
    let ((_dir, root, _engine, registry), _cwd) = setup_in_process().await;
    let id = page_create(
        &registry,
        "regression/concurrent-update",
        "Concurrent Update",
        "Body.",
    )
    .await;

    let handles: Vec<_> = ["note-one", "note-two"]
        .into_iter()
        .map(|note| {
            let registry = registry.clone();
            let id = id.clone();
            std::thread::spawn(move || {
                registry.dispatch(
                    "wm_page",
                    json!({ "action": "update", "id": id, "append_notes": note }),
                )
            })
        })
        .collect();

    for handle in handles {
        let outcome = handle.join().expect("update thread must not panic");
        let out = outcome.expect("concurrent update must succeed");
        assert_eq!(
            out.get("status").and_then(|v| v.as_str()),
            Some("updated"),
            "concurrent update must report success: {out}"
        );
    }

    let content = std::fs::read_to_string(root.join(".wm/wiki/regression/concurrent-update.md"))
        .expect("page on disk");
    let (fm, _body) = wm_engine::parser::extract_frontmatter(&content);
    assert!(fm.is_some(), "final file must parse:\n{content}");
    for note in ["note-one", "note-two"] {
        assert!(
            content.contains(note),
            "update {note} must land:\n{content}"
        );
    }
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
    let (fm, _body) = wm_engine::parser::extract_frontmatter(&content);
    let fm = fm.expect("frontmatter parses after tag update");
    assert_eq!(
        fm.tags,
        vec!["a".to_string(), "b".to_string()],
        "tags must persist, got:\n{content}"
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

fn frontmatter_region(content: &str) -> &str {
    let rest = content.strip_prefix("---\n").unwrap_or(content);
    match rest.find("\n---") {
        Some(end) => &rest[..end],
        None => rest,
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn page_get_flags_prose_body_on_record_bearing_type() {
    let ((_dir, root, _engine, registry), _cwd) = setup_in_process().await;
    std::fs::write(
        root.join(".wm/wiki/concepts/prose-record.md"),
        "---\ntitle: Prose Record\ntype: concept\nid: \"wiki:concepts:prose-record\"\nstatus: draft\n---\n\n## Context\n\nThis is prose, not a record.\n",
    )
    .expect("write legacy prose concept");
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
    assert!(
        !content.contains("schema_version"),
        "raw YAML must not leak"
    );
    assert!(out.get("record").is_some(), "parsed record must be present");
}

const RULE_RECORD_BODY: &str = "schema_version: 1\nstate: |-\n  ## Rule\n\n  Always run cargo test before commit.\nquestions:\n  - id: enforcement\n    type: choice\n    instructions: How is this rule enforced?\n    options: [ci-enforced, tool-enforced, review-enforced, convention-only]\n  - id: severity\n    type: score\n    instructions: How severe is a violation of this rule?\n    levels: [advisory, recommended, required, blocking]\n  - id: has_exception\n    type: noul\n    instructions: This rule has documented exceptions.\n  - id: applies_to\n    type: choice\n    multi: true\n    instructions: Which surfaces does this rule apply to?\n    options: [code, tests, docs, configuration, workflow, security]\nanswers: {}\n";

#[tokio::test(flavor = "multi_thread")]
async fn page_get_returns_state_for_rule() {
    let ((_dir, _root, _engine, registry), _cwd) = setup_in_process().await;
    page_create(
        &registry,
        "rules/always-test",
        "Always Test",
        RULE_RECORD_BODY,
    )
    .await;
    let out = call_ok(
        &registry,
        "wm_page",
        json!({ "action": "get", "id": "wiki:rules:always-test" }),
    )
    .await;
    let content = out.get("content").and_then(|v| v.as_str()).unwrap_or("");
    assert!(
        content.contains("Always run cargo test before commit."),
        "got {out}"
    );
    assert!(
        !content.contains("schema_version"),
        "raw YAML must not leak"
    );
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
    assert!(
        !content.contains("schema_version"),
        "raw YAML must not leak"
    );
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
    let ((_dir, root, _engine, registry), _cwd) = setup_in_process().await;
    std::fs::create_dir_all(root.join(".wm/wiki/rules")).expect("create rules dir");
    std::fs::write(
        root.join(".wm/wiki/rules/prose-rule.md"),
        "---\ntitle: Prose Rule\ntype: rule\nid: \"wiki:rules:prose-rule\"\nstatus: active\n---\n\n## Rule\n\nDo the thing.\n",
    )
    .expect("write legacy prose rule");
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

const RECORD_ENVELOPE_KEYS: [&str; 4] = ["schema_version", "state", "questions", "answers"];

#[tokio::test(flavor = "multi_thread")]
async fn page_update_rejects_prose_content_on_record_bearing_page() {
    let ((_dir, root, _engine, registry), _cwd) = setup_in_process().await;
    page_create(
        &registry,
        "concepts/update-prose",
        "Update Prose",
        "## Context\n\nOriginal prose.\n",
    )
    .await;
    let path = root.join(".wm/wiki/concepts/update-prose.md");
    let before = std::fs::read(&path).expect("page on disk before update");
    let err = call_err(
        &registry,
        "wm_page",
        json!({
            "action": "update",
            "id": "wiki:concepts:update-prose",
            "content": "## Context\n\nStill prose, not a record.\n",
        }),
    )
    .await;
    assert_eq!(err.code, "INVALID_PARAMS");
    for key in RECORD_ENVELOPE_KEYS {
        assert!(
            err.message.contains(key),
            "error must name `{key}`: {}",
            err.message
        );
    }
    assert!(
        err.message.contains("--content-file"),
        "error must point at --content-file: {}",
        err.message
    );
    assert!(
        err.message.contains("omit `content`"),
        "error must explain that omitting content preserves the body: {}",
        err.message
    );
    let after = std::fs::read(&path).expect("page on disk after rejected update");
    assert_eq!(
        before, after,
        "a rejected content update must leave the page file byte-identical"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn page_update_accepts_record_envelope_via_content() {
    let ((_dir, root, _engine, registry), _cwd) = setup_in_process().await;
    let revised = DECISION_STATE_BODY.replace("Chose SQLite over Postgres.", "Chose Postgres.");
    page_create(
        &registry,
        "decisions/update-envelope",
        "Update Envelope",
        DECISION_STATE_BODY,
    )
    .await;
    let out = call_ok(
        &registry,
        "wm_page",
        json!({
            "action": "update",
            "id": "wiki:decisions:update-envelope",
            "content": revised,
        }),
    )
    .await;
    assert_eq!(out.get("status").and_then(|v| v.as_str()), Some("updated"));
    assert!(
        format_warning(&out).is_none(),
        "a valid record update must not warn, got: {out}"
    );
    let content = std::fs::read_to_string(root.join(".wm/wiki/decisions/update-envelope.md"))
        .expect("updated page on disk");
    let (_frontmatter, body) = wm_engine::parser::extract_frontmatter(&content);
    let record = wm_engine::parse_record(&wm_engine::PageType::Decision, body)
        .expect("stored body must still parse as a record");
    assert_eq!(record.state, "## Context\n\nChose Postgres.");
}

#[tokio::test(flavor = "multi_thread")]
async fn page_update_without_content_preserves_body() {
    let ((_dir, root, _engine, registry), _cwd) = setup_in_process().await;
    page_create(
        &registry,
        "decisions/update-status-only",
        "Update Status Only",
        DECISION_STATE_BODY,
    )
    .await;
    let path = root.join(".wm/wiki/decisions/update-status-only.md");
    let before = std::fs::read_to_string(&path).expect("page on disk before update");
    let (_before_frontmatter, before_body) = wm_engine::parser::extract_frontmatter(&before);
    let out = call_ok(
        &registry,
        "wm_page",
        json!({
            "action": "update",
            "id": "wiki:decisions:update-status-only",
            "status": "approved",
        }),
    )
    .await;
    assert_eq!(out.get("status").and_then(|v| v.as_str()), Some("updated"));
    let after = std::fs::read_to_string(&path).expect("page on disk after update");
    assert!(after.contains("status: approved"), "got:\n{after}");
    let (_after_frontmatter, after_body) = wm_engine::parser::extract_frontmatter(&after);
    assert_eq!(
        before_body, after_body,
        "an update without content must leave the body untouched"
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
async fn doc_update_rejects_prose_body_on_record_bearing_type() {
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
    let err = call_err(
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
        err.message.contains("--content-file"),
        "wm_doc.update shares the page update contract, got: {}",
        err.message
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
    if engine.index.embedder().is_loaded() {
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
    if engine.index.embedder().is_loaded() {
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
    if engine.index.embedder().is_loaded() {
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

fn canonical_record(page_type: &wm_engine::PageType, state: &str) -> String {
    let record = wm_engine::DecisionRecord {
        schema_version: wm_engine::RECORD_SCHEMA_VERSION,
        state: state.to_owned(),
        questions: wm_engine::canonical_questions(page_type),
        answers: std::collections::BTreeMap::new(),
    };
    serde_yaml::to_string(&record).expect("record must serialize")
}

#[tokio::test(flavor = "multi_thread")]
async fn all_record_types_get_state_validate_and_parse() {
    use wm_engine::PageType;
    let cases = [
        (
            "rules/state-rule",
            PageType::Rule,
            "Rule state zzrulezz text.",
        ),
        (
            "memory/state-mem",
            PageType::Memory,
            "Memory state zzmemzz text.",
        ),
        (
            "tasks/state-task",
            PageType::Task,
            "Task state zztaskzz text.",
        ),
        (
            "specs/state-spec",
            PageType::Spec,
            "Spec state zzspeczz text.",
        ),
        (
            "core/state-core",
            PageType::Core,
            "Core state zzcorezz text.",
        ),
        (
            "notes/state-note",
            PageType::Note,
            "Note state zznotezz text.",
        ),
    ];
    let ((_dir, _root, _engine, registry), _cwd) = setup_in_process().await;
    for (path, page_type, state) in cases.iter() {
        page_create(
            &registry,
            path,
            "State Page",
            &canonical_record(page_type, state),
        )
        .await;
    }
    rebuild(&registry).await;

    for (path, _page_type, state) in cases.iter() {
        let dir = path.split('/').next().unwrap_or_default();
        let stem = path.rsplit('/').next().unwrap_or_default();
        let id = format!("wiki:{dir}:{stem}");
        let got = call_ok(&registry, "wm_page", json!({ "action": "get", "id": id })).await;
        assert_eq!(got["content"].as_str(), Some(*state), "content for {path}");
        assert!(got.get("record").is_some(), "record must parse for {path}");
        let validated = call_ok(&registry, "wm_validate.check", json!({ "entity": id })).await;
        assert_eq!(validated["total_errors"], 0, "validate {id}: {validated}");
    }
}

async fn validate_record_body(
    registry: &ToolRegistry,
    path: &str,
    body: &str,
) -> serde_json::Value {
    let root = std::env::current_dir().expect("project cwd");
    let (dir, stem) = path.split_once('/').unwrap_or(("concepts", path));
    let page_type = wm_engine::PageType::from_dir_name(dir).unwrap_or(wm_engine::PageType::Concept);
    let page_dir = root.join(".wm/wiki").join(dir);
    std::fs::create_dir_all(&page_dir).expect("create page dir");
    let content = format!(
        "---\ntitle: Record Page\ntype: {}\nid: \"wiki:{dir}:{stem}\"\nstatus: draft\n---\n\n{body}",
        page_type.as_str()
    );
    std::fs::write(page_dir.join(format!("{stem}.md")), content).expect("write record page");
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
    let ((_dir, root, _engine, registry), _cwd) = setup_in_process().await;
    std::fs::create_dir_all(root.join(".wm/wiki/rules")).expect("create rules dir");
    std::fs::write(
        root.join(".wm/wiki/rules/prose-rule.md"),
        "---\ntitle: Prose Rule\ntype: rule\nid: \"wiki:rules:prose-rule\"\nstatus: active\n---\n\n## Rule\n\nDo the thing.\n",
    )
    .expect("write prose rule");
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
        "tags:",
        "acceptance_criteria:",
        "text: AC1",
        "implementation_plan: plan here",
        "implementation_notes: worked on it",
    ] {
        assert!(
            content.contains(needle),
            "missing {needle:?} in:\n{content}"
        );
    }
    assert!(
        !frontmatter_region(&content).contains("{}"),
        "no '{{}}' block may be emitted, got:\n{content}"
    );

    let (fm, body) = wm_engine::parser::extract_frontmatter(&content);
    let fm = fm.expect("frontmatter parses");
    assert_eq!(fm.title, "Roundtrip Task V2");
    assert_eq!(fm.page_type.as_str(), "task");
    assert_eq!(fm.status.as_str(), "in-progress");
    assert_eq!(fm.priority.as_ref().map(|p| p.as_str()), Some("urgent"));
    assert_eq!(fm.tags, vec!["alpha", "beta", "gamma"]);
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
async fn task_check_ac_persists_to_disk() {
    let ((_dir, root, _engine, registry), _cwd) = setup_in_process().await;
    let created = call_ok(
        &registry,
        "wm_task",
        json!({
            "action": "create",
            "title": "Check AC Task",
            "status": "todo",
            "acceptance_criteria": ["first", "second", "third"],
        }),
    )
    .await;
    let id = created
        .get("id")
        .and_then(|v| v.as_str())
        .expect("task id")
        .to_string();
    let page_path = root.join(".wm/wiki/tasks/check-ac-task.md");

    let checked = call_ok(
        &registry,
        "wm_task",
        json!({ "action": "check_ac", "id": id, "index": 1 }),
    )
    .await;
    assert_eq!(checked.get("checked"), Some(&json!([1])));

    let content = std::fs::read_to_string(&page_path).expect("task file on disk");
    let (fm, _) = wm_engine::parser::extract_frontmatter(&content);
    let fm = fm.expect("frontmatter parses");
    assert_eq!(
        fm.task_data
            .as_ref()
            .expect("task data")
            .acceptance_criteria
            .len(),
        3
    );
    assert!(
        !fm.task_data
            .as_ref()
            .expect("task data")
            .acceptance_criteria[0]
            .checked,
        "first must stay unchecked:\n{content}"
    );
    assert!(
        fm.task_data
            .as_ref()
            .expect("task data")
            .acceptance_criteria[1]
            .checked,
        "second must be checked:\n{content}"
    );
    assert!(
        !fm.task_data
            .as_ref()
            .expect("task data")
            .acceptance_criteria[2]
            .checked,
        "third must stay unchecked:\n{content}"
    );

    call_ok(
        &registry,
        "wm_task",
        json!({ "action": "check_ac", "id": id, "index": 0 }),
    )
    .await;
    let content = std::fs::read_to_string(&page_path).expect("task file on disk");
    let (fm, _) = wm_engine::parser::extract_frontmatter(&content);
    let fm = fm.expect("frontmatter parses");
    assert!(
        fm.task_data
            .as_ref()
            .expect("task data")
            .acceptance_criteria[0]
            .checked,
        "index 0 must be persisted as checked:\n{content}"
    );

    call_ok(
        &registry,
        "wm_task",
        json!({ "action": "uncheck_ac", "id": id, "index": 1 }),
    )
    .await;
    let content = std::fs::read_to_string(&page_path).expect("task file on disk");
    let (fm, _) = wm_engine::parser::extract_frontmatter(&content);
    let fm = fm.expect("frontmatter parses");
    assert!(
        !fm.task_data
            .as_ref()
            .expect("task data")
            .acceptance_criteria[1]
            .checked,
        "uncheck_ac must persist checked false:\n{content}"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn task_check_ac_out_of_range_errors_without_writing() {
    let ((_dir, root, _engine, registry), _cwd) = setup_in_process().await;
    let created = call_ok(
        &registry,
        "wm_task",
        json!({
            "action": "create",
            "title": "Out Of Range AC",
            "status": "todo",
            "acceptance_criteria": ["only"],
        }),
    )
    .await;
    let id = created
        .get("id")
        .and_then(|v| v.as_str())
        .expect("task id")
        .to_string();
    let page_path = root.join(".wm/wiki/tasks/out-of-range-ac.md");

    let before = std::fs::read_to_string(&page_path).expect("task file on disk");
    let err = call_err(
        &registry,
        "wm_task",
        json!({ "action": "check_ac", "id": id, "index": 5 }),
    )
    .await;
    assert_eq!(
        err.code, "INVALID_PARAMS",
        "out-of-range must be invalid params, got {}",
        err.message
    );

    let after = std::fs::read_to_string(&page_path).expect("task file on disk");
    assert_eq!(
        before, after,
        "out-of-range index must not rewrite the page"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn task_update_status_and_assignee_persist_to_disk() {
    let ((_dir, root, _engine, registry), _cwd) = setup_in_process().await;
    let created = call_ok(
        &registry,
        "wm_task",
        json!({
            "action": "create",
            "title": "Status Assignee Task",
            "status": "todo",
        }),
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
        json!({ "action": "update", "id": id, "status": "in-progress", "assignee": "alice" }),
    )
    .await;

    let content = std::fs::read_to_string(root.join(".wm/wiki/tasks/status-assignee-task.md"))
        .expect("task file on disk");
    let (fm, _) = wm_engine::parser::extract_frontmatter(&content);
    let fm = fm.expect("frontmatter parses");
    assert_eq!(
        fm.status.as_str(),
        "in-progress",
        "status must persist to the page:\n{content}"
    );
    assert_eq!(
        fm.assignee.as_deref(),
        Some("alice"),
        "assignee must persist to the page:\n{content}"
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
    assert!(!frontmatter_region(&content).contains("{}"));
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
        "tags:",
        "relates_to:",
        "implements",
        "wiki:specs:link-update-target",
    ] {
        assert!(
            content.contains(needle),
            "missing {needle:?} in:\n{content}"
        );
    }
    assert!(
        !frontmatter_region(&content).contains("{}"),
        "no '{{}}' block, got:\n{content}"
    );
    let (fm, _body) = wm_engine::parser::extract_frontmatter(&content);
    let fm = fm.expect("frontmatter parses after link + update");
    assert_eq!(
        fm.tags,
        vec!["regression".to_string(), "linked".to_string()],
        "tags must persist, got:\n{content}"
    );

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
async fn link_sequential_keeps_both_edges() {
    let ((_dir, root, _engine, registry), _cwd) = setup_in_process().await;
    let source = page_create(
        &registry,
        "concepts/link-seq-source",
        "Link Seq Source",
        "Body.",
    )
    .await;
    page_create(&registry, "concepts/link-seq-a", "Link Seq A", "A.").await;
    page_create(&registry, "concepts/link-seq-b", "Link Seq B", "B.").await;

    for target in ["wiki:concepts:link-seq-a", "wiki:concepts:link-seq-b"] {
        call_ok(
            &registry,
            "wm_page",
            json!({
                "action": "link",
                "id": source,
                "target": target,
                "edge_type": "references",
            }),
        )
        .await;
    }

    let content = std::fs::read_to_string(root.join(".wm/wiki/concepts/link-seq-source.md"))
        .expect("source file on disk");
    assert!(
        content.contains("wiki:concepts:link-seq-a"),
        "first edge missing:\n{content}"
    );
    assert!(
        content.contains("wiki:concepts:link-seq-b"),
        "second edge missing:\n{content}"
    );
    assert_eq!(
        frontmatter_region(&content).matches("references").count(),
        2,
        "expected exactly two reference edges:\n{content}"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn link_preserves_update_authored_edge() {
    let ((_dir, root, _engine, registry), _cwd) = setup_in_process().await;
    let source = page_create(
        &registry,
        "concepts/link-merge-source",
        "Link Merge Source",
        "Body.",
    )
    .await;
    page_create(
        &registry,
        "concepts/link-merge-authored",
        "Link Merge Authored",
        "A.",
    )
    .await;
    page_create(
        &registry,
        "concepts/link-merge-linked",
        "Link Merge Linked",
        "B.",
    )
    .await;

    call_ok(
        &registry,
        "wm_page",
        json!({
            "action": "update",
            "id": source,
            "relates_to": [
                { "type": "implements", "target": "wiki:concepts:link-merge-authored" }
            ],
        }),
    )
    .await;

    call_ok(
        &registry,
        "wm_page",
        json!({
            "action": "link",
            "id": source,
            "target": "wiki:concepts:link-merge-linked",
            "edge_type": "references",
        }),
    )
    .await;

    let content = std::fs::read_to_string(root.join(".wm/wiki/concepts/link-merge-source.md"))
        .expect("source file on disk");
    assert!(
        content.contains("wiki:concepts:link-merge-authored"),
        "update-authored edge was clobbered:\n{content}"
    );
    assert!(
        content.contains("wiki:concepts:link-merge-linked"),
        "linked edge missing:\n{content}"
    );
    assert_eq!(
        content.matches("implements").count(),
        1,
        "expected the authored edge once:\n{content}"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn link_parallel_keeps_all_edges() {
    let ((_dir, root, _engine, registry), _cwd) = setup_in_process().await;
    let source = page_create(
        &registry,
        "concepts/link-par-source",
        "Link Par Source",
        "Body.",
    )
    .await;

    const TARGET_COUNT: usize = 5;
    let mut targets = Vec::new();
    for i in 0..TARGET_COUNT {
        let path = format!("concepts/link-par-{i}");
        page_create(&registry, &path, &format!("Link Par {i}"), "Body.").await;
        targets.push(format!("wiki:concepts:link-par-{i}"));
    }

    let mut handles = Vec::new();
    for target in &targets {
        let reg = registry.clone();
        let id = source.clone();
        let target = target.clone();
        handles.push(tokio::spawn(async move {
            call(
                &reg,
                "wm_page",
                json!({
                    "action": "link",
                    "id": id,
                    "target": target,
                    "edge_type": "references",
                }),
            )
            .await
        }));
    }

    for handle in handles {
        match handle.await.expect("join link task") {
            Ok(_) => {}
            Err(e) => panic!("parallel link failed: {} ({})", e.message, e.code),
        }
    }

    let content = std::fs::read_to_string(root.join(".wm/wiki/concepts/link-par-source.md"))
        .expect("source file on disk");
    for target in &targets {
        assert!(content.contains(target), "missing {target} in:\n{content}");
    }
    assert_eq!(
        frontmatter_region(&content).matches("references").count(),
        TARGET_COUNT,
        "expected {TARGET_COUNT} reference edges:\n{content}"
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
async fn memory_add_honors_explicit_id() {
    let ((_dir, root, _engine, registry), _cwd) = setup_in_process().await;
    let out = call_ok(
        &registry,
        "wm_memory",
        json!({
            "action": "add",
            "id": "wiki:memory:explicit-id-memory",
            "title": "Explicit Id Memory",
            "content": "Durable knowledge keyed by an explicit id.",
        }),
    )
    .await;
    assert_eq!(
        out.get("id").and_then(|v| v.as_str()),
        Some("wiki:memory:explicit-id-memory"),
        "add must echo the explicit id: {out}"
    );

    let path = memory_page_path(&root, "explicit-id-memory");
    assert!(
        path.exists(),
        "explicit id must drive the page path: {}",
        path.display()
    );

    let page = call_ok(
        &registry,
        "wm_memory",
        json!({ "action": "get", "id": "wiki:memory:explicit-id-memory" }),
    )
    .await;
    assert_eq!(
        page.get("id").and_then(|v| v.as_str()),
        Some("wiki:memory:explicit-id-memory"),
        "stored page must be readable by the explicit id: {page}"
    );
    assert!(
        page.get("content")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .contains("explicit id"),
        "stored content must round-trip: {page}"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn memory_add_rejects_mismatched_explicit_id() {
    let ((_dir, root, _engine, registry), _cwd) = setup_in_process().await;
    let err = call_err(
        &registry,
        "wm_memory",
        json!({
            "action": "add",
            "id": "wiki:memory:other-slug",
            "title": "Mismatch Id",
            "content": "Must never be stored under a different slug.",
        }),
    )
    .await;

    assert_eq!(err.code, "INVALID_PARAMS", "mismatched id must be rejected");
    assert!(
        err.message.contains("other-slug"),
        "error must name the given id: {}",
        err.message
    );
    assert!(
        err.message.contains("mismatch-id"),
        "error must name the title slug: {}",
        err.message
    );
    assert!(!memory_page_path(&root, "other-slug").exists());
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
    let (_frontmatter, body) = wm_engine::parser::extract_frontmatter(&content);
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
    let (fm, _body) = wm_engine::parser::extract_frontmatter(&content);
    let fm = fm.expect("task frontmatter must parse back");
    assert_eq!(
        fm.title.as_str(),
        nasty_title,
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
    let (sub_fm, _sub_body) = wm_engine::parser::extract_frontmatter(&sub_content);
    let sub_fm = sub_fm.expect("subtask frontmatter must parse back");
    assert_eq!(
        sub_fm.title.as_str(),
        nasty_title,
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

#[tokio::test(flavor = "multi_thread")]
async fn page_create_applies_extra_frontmatter_on_creation() {
    let ((_dir, root, _engine, registry), _cwd) = setup_in_process().await;
    let created = call_ok(
        &registry,
        "wm_page",
        json!({
            "action": "create",
            "path": "specs/create-extra-fm",
            "title": "Create Extra FM",
            "type": "spec",
            "content": "Spec body.",
            "extra_frontmatter": {
                "acceptance_criteria": ["AC one", "AC two"],
                "stakeholders": ["alice"],
            },
        }),
    )
    .await;
    assert!(
        created.get("id").and_then(|v| v.as_str()).is_some(),
        "create must return an id, got {created}"
    );

    let content = std::fs::read_to_string(root.join(".wm/wiki/specs/create-extra-fm.md"))
        .expect("created page on disk");
    let (fm, _body) = wm_engine::parser::extract_frontmatter(&content);
    let fm = fm.expect("frontmatter parses after create with extra keys");
    assert_eq!(
        fm.task_data
            .as_ref()
            .expect("task data")
            .acceptance_criteria
            .len(),
        2,
        "acceptance_criteria must be applied on create:\n{content}"
    );
    assert_eq!(
        fm.task_data
            .as_ref()
            .expect("task data")
            .acceptance_criteria[0]
            .text,
        "AC one"
    );
    assert_eq!(
        fm.spec_data
            .as_ref()
            .expect("spec data")
            .stakeholders
            .clone(),
        vec!["alice".to_string()],
        "stakeholders must be applied on create:\n{content}"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn task_create_persists_priority_and_acceptance_criteria() {
    let ((_dir, root, _engine, registry), _cwd) = setup_in_process().await;
    call_ok(
        &registry,
        "wm_task",
        json!({
            "action": "create",
            "title": "Probe D Task",
            "priority": "high",
            "acceptance_criteria": ["AC-D1", "AC-D2"],
        }),
    )
    .await;

    let content = std::fs::read_to_string(root.join(".wm/wiki/tasks/probe-d-task.md"))
        .expect("created task on disk");
    let (fm, _body) = wm_engine::parser::extract_frontmatter(&content);
    let fm = fm.expect("task frontmatter parses");
    assert_eq!(
        fm.priority.as_ref().map(|p| p.as_str()),
        Some("high"),
        "priority lost:\n{content}"
    );
    assert_eq!(
        fm.task_data
            .as_ref()
            .expect("task data")
            .acceptance_criteria
            .len(),
        2,
        "acceptance_criteria must be applied on task create:\n{content}"
    );
    assert_eq!(
        fm.task_data
            .as_ref()
            .expect("task data")
            .acceptance_criteria[0]
            .text,
        "AC-D1"
    );
    assert_eq!(
        fm.task_data
            .as_ref()
            .expect("task data")
            .acceptance_criteria[1]
            .text,
        "AC-D2"
    );

    let validated = call_ok(
        &registry,
        "wm_validate.check",
        json!({ "entity": "wiki:tasks:probe-d-task" }),
    )
    .await;
    let missing_ac = validated
        .get("errors")
        .and_then(|v| v.as_array())
        .map(|errors| {
            errors.iter().any(|error| {
                error.get("field").and_then(|v| v.as_str()) == Some("acceptance_criteria")
            })
        })
        .unwrap_or(false);
    assert!(
        !missing_ac,
        "a task created with ACs must not trip the missing-AC rule: {validated}"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn memory_add_emits_record_body() {
    let ((_dir, root, _engine, registry), _cwd) = setup_in_process().await;
    call_ok(
        &registry,
        "wm_memory",
        json!({
            "action": "add",
            "title": "Record Memory",
            "content": "## Memory\n\nDurable knowledge that must survive.",
        }),
    )
    .await;

    let content = std::fs::read_to_string(root.join(".wm/wiki/memory/record-memory.md"))
        .expect("created memory on disk");
    let (_fm, body) = wm_engine::parser::extract_frontmatter(&content);
    assert!(
        body.trim_start().starts_with("schema_version: 1"),
        "memory add must emit a record body, got:\n{body}"
    );
    let record = wm_engine::parse_record(&wm_engine::PageType::Memory, body)
        .expect("memory body must parse as a memory record");
    assert!(
        record
            .state
            .contains("Durable knowledge that must survive."),
        "memory prose must become the state payload:\n{body}"
    );
}

fn set_memory_dedup_threshold(engine: &wm_core::engine::EngineState, value: f64) {
    engine
        .config
        .write()
        .expect("config lock")
        .search
        .scoring
        .memory_dedup_threshold = value;
}

fn memory_page_path(root: &std::path::Path, slug: &str) -> std::path::PathBuf {
    root.join(".wm/wiki/memory").join(format!("{slug}.md"))
}

fn memory_record_answers(
    root: &std::path::Path,
    slug: &str,
) -> std::collections::BTreeMap<String, wm_engine::AnswerValue> {
    let raw = std::fs::read_to_string(memory_page_path(root, slug)).expect("memory page on disk");
    let (_fm, body) = wm_engine::parser::extract_frontmatter(&raw);
    wm_engine::parse_record(&wm_engine::PageType::Memory, body)
        .expect("memory body parses as a record")
        .answers
}

#[tokio::test(flavor = "multi_thread")]
async fn memory_add_skips_existing_page_and_writes_nothing() {
    let ((_dir, root, _engine, registry), _cwd) = setup_in_process().await;
    let content = "Durable knowledge that must survive a rebuild.";
    call_ok(
        &registry,
        "wm_memory",
        json!({ "action": "add", "title": "Dedup Canon", "content": content }),
    )
    .await;
    let path = memory_page_path(&root, "dedup-canon");
    let before = std::fs::read_to_string(&path).expect("first add writes the page");

    let out = call_ok(
        &registry,
        "wm_memory",
        json!({ "action": "add", "title": "Dedup Canon", "content": content }),
    )
    .await;

    assert_eq!(out["status"], "skipped", "duplicate add must skip: {out}");
    assert!(
        out["reason"]
            .as_str()
            .is_some_and(|reason| !reason.is_empty()),
        "skipped add must carry a reason: {out}"
    );
    assert_eq!(out["nearest"]["id"], "wiki:memory:dedup-canon");
    assert_eq!(
        std::fs::read_to_string(&path).expect("page still present"),
        before,
        "a skipped add must not rewrite the page"
    );
    assert_eq!(
        std::fs::read_dir(root.join(".wm/wiki/memory"))
            .expect("memory dir")
            .count(),
        1,
        "a skipped add must not create a new file"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn memory_add_skips_near_duplicate_score_without_writing() {
    let ((_dir, root, engine, registry), _cwd) = setup_in_process().await;
    let content = "Always run the full test suite before pushing a commit.";
    call_ok(
        &registry,
        "wm_memory",
        json!({ "action": "add", "title": "Score Canon", "content": content }),
    )
    .await;

    set_memory_dedup_threshold(&engine, 0.01);
    let out = call_ok(
        &registry,
        "wm_memory",
        json!({ "action": "add", "title": "Score Copy", "content": content }),
    )
    .await;

    assert_eq!(out["status"], "skipped", "near-duplicate must skip: {out}");
    assert_eq!(out["nearest"]["id"], "wiki:memory:score-canon");
    assert!(
        out["nearest"]["score"].is_number(),
        "score evidence must be numeric: {out}"
    );
    assert!(
        !memory_page_path(&root, "score-copy").exists(),
        "a skipped add must not write the target page"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn memory_add_populates_all_answers_for_a_novel_entry() {
    let ((_dir, root, _engine, registry), _cwd) = setup_in_process().await;
    let out = call_ok(
        &registry,
        "wm_memory",
        json!({
            "action": "add",
            "title": "Novel Answer",
            "content": "Prefer line-based edits; see wiki:specs:memory-write-decision-gate.",
        }),
    )
    .await;

    assert_eq!(out["status"], "created", "novel add must store: {out}");
    assert!(
        out["nearest"].is_object(),
        "evidence must be returned on store too: {out}"
    );
    assert!(
        out["nearest"]
            .as_object()
            .is_some_and(|nearest| nearest.contains_key("id")),
        "nearest must expose an id slot: {out}"
    );

    let answers = memory_record_answers(&root, "novel-answer");
    assert_eq!(
        answers.get("layer"),
        Some(&wm_engine::AnswerValue::Label("project".to_owned()))
    );
    assert_eq!(
        answers.get("store_or_skip"),
        Some(&wm_engine::AnswerValue::Bool(true))
    );
    assert_eq!(
        answers.get("dedup_action"),
        Some(&wm_engine::AnswerValue::Label("new".to_owned()))
    );
    assert_eq!(
        answers.get("confidence"),
        Some(&wm_engine::AnswerValue::Label("high".to_owned()))
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn memory_add_session_layer_bypasses_the_gate() {
    let ((_dir, root, engine, registry), _cwd) = setup_in_process().await;
    let out = call_ok(
        &registry,
        "wm_memory",
        json!({
            "action": "add",
            "title": "Session Note",
            "content": "Short lived scratch note.",
            "layer": "session",
        }),
    )
    .await;

    assert_eq!(out["status"], "created");
    assert_eq!(out["layer"], "session");
    assert_eq!(
        engine.session_memory.count(),
        1,
        "session store must hold it"
    );
    assert!(
        !memory_page_path(&root, "session-note").exists(),
        "session layer must not write a wiki page"
    );
    assert!(
        out.get("nearest").is_none(),
        "session layer is exempt from the gate: {out}"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn memory_add_confidence_high_when_anchor_cited() {
    let ((_dir, root, _engine, registry), _cwd) = setup_in_process().await;
    call_ok(
        &registry,
        "wm_memory",
        json!({
            "action": "add",
            "title": "Anchored Memory",
            "content": "Edit apps/wm-core/src/lib.rs and re-run the suite.",
        }),
    )
    .await;
    assert_eq!(
        memory_record_answers(&root, "anchored-memory")
            .get("confidence")
            .cloned(),
        Some(wm_engine::AnswerValue::Label("high".to_owned()))
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn memory_add_confidence_medium_without_anchor() {
    let ((_dir, root, _engine, registry), _cwd) = setup_in_process().await;
    call_ok(
        &registry,
        "wm_memory",
        json!({
            "action": "add",
            "title": "Unanchored Memory",
            "content": "Prefer line-based YAML edits for durable memory.",
        }),
    )
    .await;
    assert_eq!(
        memory_record_answers(&root, "unanchored-memory")
            .get("confidence")
            .cloned(),
        Some(wm_engine::AnswerValue::Label("medium".to_owned()))
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn memory_add_threshold_override_changes_skip_behaviour() {
    let ((_dir, root, engine, registry), _cwd) = setup_in_process().await;
    let content = "Always run the full test suite before pushing a commit.";
    call_ok(
        &registry,
        "wm_memory",
        json!({ "action": "add", "title": "Override Canon", "content": content }),
    )
    .await;

    set_memory_dedup_threshold(&engine, f64::MAX);
    let stored = call_ok(
        &registry,
        "wm_memory",
        json!({ "action": "add", "title": "Override Copy", "content": content }),
    )
    .await;
    assert_eq!(
        stored["status"], "created",
        "an unreachable threshold must allow the write: {stored}"
    );
    assert!(memory_page_path(&root, "override-copy").exists());

    set_memory_dedup_threshold(&engine, 0.01);
    let skipped = call_ok(
        &registry,
        "wm_memory",
        json!({ "action": "add", "title": "Override Twin", "content": content }),
    )
    .await;
    assert_eq!(
        skipped["status"], "skipped",
        "a reachable threshold must reject the near-duplicate: {skipped}"
    );
    assert!(!memory_page_path(&root, "override-twin").exists());
}

#[tokio::test(flavor = "multi_thread")]
async fn memory_add_dedup_action_never_emits_merge_or_supersede() {
    let ((_dir, root, _engine, registry), _cwd) = setup_in_process().await;
    call_ok(
        &registry,
        "wm_memory",
        json!({ "action": "add", "title": "Vocabulary Probe", "content": "Novel entry." }),
    )
    .await;

    let answer = memory_record_answers(&root, "vocabulary-probe")
        .get("dedup_action")
        .cloned();
    assert_eq!(
        answer,
        Some(wm_engine::AnswerValue::Label("new".to_owned())),
        "v1 only emits new|skip"
    );
    assert!(
        !matches!(
            answer,
            Some(wm_engine::AnswerValue::Label(ref label))
                if label == "merge" || label == "supersede"
        ),
        "merge and supersede are deferred out of v1"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn memory_add_stores_when_the_index_is_empty() {
    let ((_dir, root, _engine, registry), _cwd) = setup_in_process().await;
    let out = call_ok(
        &registry,
        "wm_memory",
        json!({ "action": "add", "title": "First Memory", "content": "Brand new knowledge." }),
    )
    .await;

    assert_eq!(out["status"], "created", "empty index must store: {out}");
    assert!(
        out["nearest"]["id"].is_null(),
        "empty index yields no nearest evidence: {out}"
    );
    assert!(memory_page_path(&root, "first-memory").exists());
}

#[tokio::test(flavor = "multi_thread")]
async fn memory_add_returns_below_threshold_evidence_on_store() {
    let ((_dir, root, engine, registry), _cwd) = setup_in_process().await;
    let content = "Always run the full test suite before pushing a commit.";
    call_ok(
        &registry,
        "wm_memory",
        json!({ "action": "add", "title": "Below Canon", "content": content }),
    )
    .await;

    set_memory_dedup_threshold(&engine, 1.0e9);
    let out = call_ok(
        &registry,
        "wm_memory",
        json!({ "action": "add", "title": "Below Probe", "content": content }),
    )
    .await;

    assert_eq!(
        out["status"], "created",
        "below threshold must store: {out}"
    );
    assert_eq!(
        out["nearest"]["id"], "wiki:memory:below-canon",
        "positive nearest must be reported even on store: {out}"
    );
    assert!(
        out["nearest"]["score"]
            .as_f64()
            .is_some_and(|score| score > 0.0),
        "nearest score must be a positive number: {out}"
    );
    assert!(memory_page_path(&root, "below-probe").exists());
}

#[tokio::test(flavor = "multi_thread")]
async fn memory_add_rejects_whitespace_title() {
    let ((_dir, root, _engine, registry), _cwd) = setup_in_process().await;
    let err = call_err(
        &registry,
        "wm_memory",
        json!({ "action": "add", "title": "   ", "content": "Valid content." }),
    )
    .await;

    assert_eq!(err.code, "INVALID_PARAMS", "blank title must be rejected");
    let written = std::fs::read_dir(root.join(".wm/wiki/memory"))
        .map(|dir| dir.count())
        .unwrap_or(0);
    assert_eq!(written, 0, "a rejected add must not write a page");
}

#[tokio::test(flavor = "multi_thread")]
async fn memory_add_rejects_empty_content() {
    let ((_dir, _root, _engine, registry), _cwd) = setup_in_process().await;
    let empty = call_err(
        &registry,
        "wm_memory",
        json!({ "action": "add", "title": "Empty Content", "content": "" }),
    )
    .await;
    assert_eq!(
        empty.code, "INVALID_PARAMS",
        "empty content must be rejected"
    );

    let blank = call_err(
        &registry,
        "wm_memory",
        json!({ "action": "add", "title": "Blank Content", "content": "   \n" }),
    )
    .await;
    assert_eq!(
        blank.code, "INVALID_PARAMS",
        "blank content must be rejected"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn memory_add_rejects_the_unimplemented_global_layer() {
    let ((_dir, root, _engine, registry), _cwd) = setup_in_process().await;
    let err = call_err(
        &registry,
        "wm_memory",
        json!({
            "action": "add",
            "title": "Global Note",
            "content": "Must not silently land in project memory.",
            "layer": "global",
        }),
    )
    .await;

    assert_eq!(err.code, "INVALID_PARAMS", "global layer must be rejected");
    let written = std::fs::read_dir(root.join(".wm/wiki/memory"))
        .map(|dir| dir.count())
        .unwrap_or(0);
    assert_eq!(written, 0, "a rejected global add must not write a page");
}

#[test]
fn memory_record_layer_accepts_the_reserved_low_confidence() {
    let mut record = wm_engine::DecisionRecord {
        schema_version: wm_engine::RECORD_SCHEMA_VERSION,
        state: "Reserved low confidence seat.".to_owned(),
        questions: wm_engine::canonical_questions(&wm_engine::PageType::Memory),
        answers: std::collections::BTreeMap::new(),
    };
    record.answers.insert(
        "layer".to_owned(),
        wm_engine::AnswerValue::Label("project".to_owned()),
    );
    record.answers.insert(
        "store_or_skip".to_owned(),
        wm_engine::AnswerValue::Bool(true),
    );
    record.answers.insert(
        "dedup_action".to_owned(),
        wm_engine::AnswerValue::Label("skip".to_owned()),
    );
    record.answers.insert(
        "confidence".to_owned(),
        wm_engine::AnswerValue::Label("low".to_owned()),
    );

    assert!(
        wm_engine::validate_record(&wm_engine::PageType::Memory, &record).is_ok(),
        "the record layer must accept low as a confidence value"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn memory_tool_description_documents_the_skipped_status() {
    let ((_dir, _root, _engine, registry), _cwd) = setup_in_process().await;
    let tools = registry.list_tools();
    let memory = tools
        .iter()
        .find(|tool| tool["name"] == "wm_memory")
        .expect("wm_memory must be registered");
    let description = memory["description"].as_str().unwrap_or_default();
    assert!(
        description.contains("skipped"),
        "wm_memory description must document the skipped status: {description}"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn page_create_record_bearing_type_writes_record_body() {
    let ((_dir, root, _engine, registry), _cwd) = setup_in_process().await;
    page_create(
        &registry,
        "concepts/record-body",
        "Record Body",
        "## Context\n\nProse goes into the state.\n",
    )
    .await;

    let content = std::fs::read_to_string(root.join(".wm/wiki/concepts/record-body.md"))
        .expect("created page on disk");
    let (_fm, body) = wm_engine::parser::extract_frontmatter(&content);
    assert!(
        body.trim_start().starts_with("schema_version: 1"),
        "body must be a v1 record, got:\n{body}"
    );
    for needle in ["state:", "questions:", "answers:"] {
        assert!(
            body.contains(needle),
            "record body missing {needle:?}:\n{body}"
        );
    }
    let record = wm_engine::parse_record(&wm_core::engine::PageType::Concept, body)
        .expect("created body must parse as a concept record");
    assert_eq!(
        record.state, "## Context\n\nProse goes into the state.",
        "prose must become the state payload"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn task_create_acceptance_criteria_round_trips_yaml_hostile_text() {
    let ((_dir, root, _engine, registry), _cwd) = setup_in_process().await;
    let hostile = "He said \"stop\": then\n[bracket] start";
    call_ok(
        &registry,
        "wm_task",
        json!({
            "action": "create",
            "title": "Hostile AC Task",
            "acceptance_criteria": [hostile],
        }),
    )
    .await;

    let content = std::fs::read_to_string(root.join(".wm/wiki/tasks/hostile-ac-task.md"))
        .expect("created task on disk");
    let (fm, _body) = wm_engine::parser::extract_frontmatter(&content);
    let fm = fm.expect("page must parse despite hostile AC text");
    assert_eq!(
        fm.task_data
            .as_ref()
            .expect("task data")
            .acceptance_criteria
            .len(),
        1,
        "hostile AC must survive:\n{content}"
    );
    assert_eq!(
        fm.task_data
            .as_ref()
            .expect("task data")
            .acceptance_criteria[0]
            .text,
        hostile,
        "AC text must round-trip intact:\n{content}"
    );
    assert!(
        !fm.task_data
            .as_ref()
            .expect("task data")
            .acceptance_criteria[0]
            .checked,
        "new AC defaults to unchecked"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn page_create_passes_through_existing_record_envelope() {
    let ((_dir, root, _engine, registry), _cwd) = setup_in_process().await;
    let envelope =
        wm_core::page::build_record_body(&wm_core::engine::PageType::Concept, "Original prose.")
            .expect("envelope renders");
    page_create(
        &registry,
        "concepts/envelope-passthrough",
        "Envelope Passthrough",
        &envelope,
    )
    .await;

    let content = std::fs::read_to_string(root.join(".wm/wiki/concepts/envelope-passthrough.md"))
        .expect("created page on disk");
    let (_fm, body) = wm_engine::parser::extract_frontmatter(&content);
    assert_eq!(
        body.trim_end(),
        envelope.trim_end(),
        "an existing envelope must be stored verbatim:\n{content}"
    );
    assert_eq!(
        body.matches("schema_version").count(),
        1,
        "the envelope must not be double-wrapped:\n{content}"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn page_update_still_applies_extra_frontmatter() {
    let ((_dir, root, _engine, registry), _cwd) = setup_in_process().await;
    let id = page_create(
        &registry,
        "regression/update-extra-fm",
        "Update Extra FM",
        "Body.",
    )
    .await;
    let out = call_ok(
        &registry,
        "wm_page",
        json!({
            "action": "update",
            "id": id,
            "extra_frontmatter": { "confidence": "high" },
        }),
    )
    .await;
    assert_eq!(out.get("status").and_then(|v| v.as_str()), Some("updated"));

    let content = std::fs::read_to_string(root.join(".wm/wiki/regression/update-extra-fm.md"))
        .expect("updated page on disk");
    assert!(
        content.contains("confidence: high"),
        "update must still apply extra keys through the shared helper:\n{content}"
    );
    let (fm, _body) = wm_engine::parser::extract_frontmatter(&content);
    assert!(fm.is_some(), "frontmatter must still parse:\n{content}");
}
