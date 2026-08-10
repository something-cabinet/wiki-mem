//! Security-hardening RED/GREEN tests for WM-001..WM-004, path confinement and
//! the shared audit sink.
//!
//! Tests dispatch through the real `ToolRegistry` in-process so the full
//! handler pipeline (schema deserialization → confinement → audit) is covered
//! without spawning daemons.

#[path = "helpers/setup.rs"]
mod setup;

use std::path::Path;
use std::sync::Arc;

use wm_core::engine::EngineState;
use wm_core::mcp::transport::ToolRegistry;
use wm_core::mcp::tools;

fn setup_in_process(
) -> (tempfile::TempDir, std::path::PathBuf, Arc<EngineState>, Arc<ToolRegistry>) {
    let (dir, root) = setup::setup_test_project();
    let config = wm_core::config::load_config(&root).unwrap_or_default();
    let (state, _audit_rx) = EngineState::new(config, root.clone());
    let engine = Arc::new(state);
    let mut registry = ToolRegistry::new();
    tools::register_all_tools(&mut registry, engine.clone());
    (dir, root, engine, Arc::new(registry))
}

async fn call(
    registry: &ToolRegistry,
    tool: &str,
    args: serde_json::Value,
) -> Result<serde_json::Value, wm_core::error::ToolError> {
    registry.dispatch_async(tool, args).await
}

fn err_contains(err: &wm_core::error::ToolError, needle: &str) -> bool {
    err.message.contains(needle)
}

// ---------------------------------------------------------------------------
// WM-001 — wm_model remove must validate against MODEL_REGISTRY
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread")]
async fn wm001_remove_traversal_name_is_rejected() {
    let (_dir, _root, _engine, registry) = setup_in_process();
    let err = call(
        &registry,
        "wm_model",
        serde_json::json!({ "action": "remove", "name": "../../../victim" }),
    )
    .await
    .expect_err("traversal model name must be rejected");
    assert!(
        err_contains(&err, "single path segment"),
        "expected a segment-validation error, got: {}",
        err.message
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn wm001_remove_unknown_model_returns_error_not_silent_success() {
    let (_dir, _root, _engine, registry) = setup_in_process();
    let err = call(
        &registry,
        "wm_model",
        serde_json::json!({ "action": "remove", "name": "definitely-not-a-model" }),
    )
    .await
    .expect_err("unknown model name must produce an explicit error");
    assert!(
        err_contains(&err, "not found") || err_contains(&err, "Unknown model"),
        "expected a clean not-found error, got: {}",
        err.message
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn wm001_remove_registry_model_still_succeeds() {
    let (_dir, _root, _engine, registry) = setup_in_process();
    let out = call(
        &registry,
        "wm_model",
        serde_json::json!({ "action": "remove", "name": "bge-small-en-v1.5" }),
    )
    .await
    .expect("registry-valid model name must be accepted");
    assert_eq!(out["status"], "removed");
}

#[test]
fn wm001_registry_is_exported_constant() {
    let names = wm_core::mcp::tools::model::MODEL_REGISTRY;
    assert_eq!(names, &["bge-small-en-v1.5", "bge-base-en-v1.5", "all-MiniLM-L6-v2"]);
}

// ---------------------------------------------------------------------------
// WM-002 — template writes must be confined; errors name the variable
//
// `wm_template run` (the runner that substituted caller variables into write
// paths) has been removed. The remaining write path is `wm_template create`,
// whose template name is caller-controlled and becomes a file under
// `.wm/templates/` — traversal names must still be confined.
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread")]
async fn wm002_template_create_traversal_name_is_rejected() {
    let (_dir, root, _engine, registry) = setup_in_process();
    let err = call(
        &registry,
        "wm_template",
        serde_json::json!({
            "action": "create",
            "name": "../../escape-create",
            "description": "must not escape",
            "content": "payload {{name}}"
        }),
    )
    .await
    .expect_err("template create with traversal name must be rejected");
    assert!(
        err_contains(&err, "offending template name:") || err_contains(&err, "Access denied")
            || err_contains(&err, "escapes"),
        "expected a confinement error naming the template, got: {}",
        err.message
    );
    assert!(
        !root
            .parent()
            .unwrap()
            .join("escape-create.json")
            .exists(),
        "no template file may be written outside .wm/templates/"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn wm002_benign_template_create_still_works() {
    let (_dir, root, _engine, registry) = setup_in_process();
    call(
        &registry,
        "wm_template",
        serde_json::json!({
            "action": "create",
            "name": "benign",
            "description": "benign create",
            "content": "payload {{name}}"
        }),
    )
    .await
    .expect("benign template create must succeed");
    assert!(
        root.join(".wm").join("templates").join("benign.json").exists(),
        "benign template should be written under .wm/templates/"
    );
}

// ---------------------------------------------------------------------------
// WM-003 — wm_page / wm_doc traversal is rejected
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread")]
async fn wm003_page_create_traversal_is_rejected() {
    let (_dir, root, _engine, registry) = setup_in_process();
    let err = call(
        &registry,
        "wm_page",
        serde_json::json!({
            "action": "create",
            "path": "../../../evil",
            "title": "Evil",
            "content": "must not land"
        }),
    )
    .await
    .expect_err("wm_page create with traversal must be rejected");
    assert!(
        err_contains(&err, "Access denied") || err_contains(&err, "escapes"),
        "expected a confinement error, got: {}",
        err.message
    );
    assert!(
        !root.parent().unwrap().join("evil.md").exists(),
        "no page may be created outside .wm/wiki/"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn wm003_doc_create_traversal_is_rejected() {
    let (_dir, root, _engine, registry) = setup_in_process();
    let err = call(
        &registry,
        "wm_doc",
        serde_json::json!({
            "action": "create",
            "path": "../../escape-doc",
            "title": "Evil",
            "content": "must not land"
        }),
    )
    .await
    .expect_err("wm_doc create with traversal must be rejected");
    assert!(
        err_contains(&err, "Access denied") || err_contains(&err, "escapes"),
        "expected a confinement error, got: {}",
        err.message
    );
    assert!(
        !root.parent().unwrap().join("escape-doc.md").exists(),
        "no doc may be created outside .wm/wiki/"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn wm003_doc_update_traversal_is_rejected() {
    let (_dir, _root, _engine, registry) = setup_in_process();
    let err = call(
        &registry,
        "wm_doc",
        serde_json::json!({
            "action": "update",
            "path": "../../escape-update",
            "title": "Evil"
        }),
    )
    .await
    .expect_err("wm_doc update with traversal must be rejected");
    assert!(
        err_contains(&err, "Access denied") || err_contains(&err, "escapes"),
        "expected a confinement error, got: {}",
        err.message
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn wm003_doc_delete_traversal_is_rejected() {
    let (_dir, _root, _engine, registry) = setup_in_process();
    let err = call(
        &registry,
        "wm_doc",
        serde_json::json!({ "action": "delete", "path": "../../escape-delete" }),
    )
    .await
    .expect_err("wm_doc delete with traversal must be rejected");
    assert!(
        err_contains(&err, "Access denied") || err_contains(&err, "escapes"),
        "expected a confinement error, got: {}",
        err.message
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn wm003_doc_create_valid_path_still_works() {
    let (_dir, root, _engine, registry) = setup_in_process();
    call(
        &registry,
        "wm_doc",
        serde_json::json!({
            "action": "create",
            "path": "reference/new-doc",
            "title": "New Doc",
            "content": "body"
        }),
    )
    .await
    .expect("valid doc create must succeed");
    assert!(
        root.join(".wm/wiki/reference/new-doc.md").exists(),
        "valid doc should be written inside .wm/wiki/"
    );
}

// ---------------------------------------------------------------------------
// WM-004 — add_source must reject /etc/hosts, .git/config and dot-files
// ---------------------------------------------------------------------------

fn source_config() -> serde_json::Value {
    serde_json::json!({
        "action": "add",
        "path": ""
    })
}

#[tokio::test(flavor = "multi_thread")]
async fn wm004_etc_hosts_is_rejected() {
    let (_dir, _root, _engine, registry) = setup_in_process();
    let mut args = source_config();
    args["path"] = serde_json::json!("/etc/hosts");
    let err = call(&registry, "wm_source", args)
        .await
        .expect_err("add_source must reject /etc/hosts");
    assert!(
        err_contains(&err, "Access denied"),
        "expected an access-denied error, got: {}",
        err.message
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn wm004_git_config_pat_exposure_is_rejected() {
    let (_dir, _root, _engine, registry) = setup_in_process();
    let mut args = source_config();
    args["path"] = serde_json::json!(".git/config");
    let err = call(&registry, "wm_source", args)
        .await
        .expect_err("add_source must reject .git/config (GitHub PAT exposure path)");
    assert!(
        err_contains(&err, "Access denied"),
        "expected an access-denied error, got: {}",
        err.message
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn wm004_dotfile_under_allowed_root_is_rejected() {
    let (_dir, root, _engine, registry) = setup_in_process();
    std::fs::create_dir_all(root.join("docs")).expect("create docs dir");
    std::fs::write(root.join("docs/.secret.md"), "secret").expect("write dotfile");
    let mut args = source_config();
    args["path"] = serde_json::json!(root.join("docs/.secret.md"));
    let err = call(&registry, "wm_source", args)
        .await
        .expect_err("add_source must reject a dot-file under an allowed source_dirs entry");
    assert!(
        err_contains(&err, "Access denied"),
        "expected an access-denied error, got: {}",
        err.message
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn wm004_allowed_source_still_ingests() {
    let (_dir, root, _engine, registry) = setup_in_process();
    std::fs::create_dir_all(root.join("docs")).expect("create docs dir");
    std::fs::write(root.join("docs/hello.md"), "# Hello").expect("write source file");
    let mut args = source_config();
    args["path"] = serde_json::json!(root.join("docs/hello.md"));
    let out = call(&registry, "wm_source", args)
        .await
        .expect("a .md file under a configured source_dirs entry must ingest");
    assert_eq!(out["state"], "pending");
    assert!(out["id"].as_str().is_some());
}

// ---------------------------------------------------------------------------
// Audit sink — a rejected operation leaves a durable, queryable audit line
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread")]
async fn audit_sink_records_path_escape_rejection() {
    let (_dir, root, _engine, registry) = setup_in_process();

    let _err = call(
        &registry,
        "wm_doc",
        serde_json::json!({
            "action": "create",
            "path": "../../audit-escape",
            "title": "Evil",
            "content": "x"
        }),
    )
    .await
    .expect_err("traversal must be rejected so the audit line is produced");

    let log_path = root.join(".wm").join("log.jsonl");
    let content = std::fs::read_to_string(&log_path)
        .unwrap_or_else(|_| panic!("audit log should exist at {}", log_path.display()));
    assert!(
        content.contains("\"kind\":\"path_escape\""),
        "audit log must contain a path_escape event, got: {}",
        content
    );
    assert!(
        content.contains("audit-escape"),
        "audit log must carry the attacker-controlled path (sanitized), got: {}",
        content
    );
    assert!(
        content.contains("\"category\":\"security\""),
        "audit event must be categorised as security, got: {}",
        content
    );

    // wm_log must be able to query the same file back.
    let out = call(
        &registry,
        "wm_log.filter",
        serde_json::json!({ "text": "path_escape", "limit": 50 }),
    )
    .await
    .expect("wm_log.filter should read the audit log");
    let total = out["total"].as_u64().unwrap_or(0);
    assert!(
        total >= 1,
        "wm_log.filter must surface the security event, got: {}",
        out
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn audit_sink_sanitizes_control_characters() {
    let (_dir, root, _engine, registry) = setup_in_process();
    let evil = "../../evil\u{0007}newline\u{000a}injected";
    let _err = call(
        &registry,
        "wm_doc",
        serde_json::json!({
            "action": "create",
            "path": evil,
            "title": "Evil",
            "content": "x"
        }),
    )
    .await
    .expect_err("traversal must be rejected");

    let log_path = root.join(".wm").join("log.jsonl");
    let content = std::fs::read_to_string(&log_path).expect("audit log written");
    let lines = content.lines().count();
    // A control-char newline must not be able to split the JSON line.
    assert_eq!(
        lines, 1,
        "control characters must be stripped so the audit line cannot be split, got: {}",
        content
    );
    assert!(
        content.contains("\"kind\":\"path_escape\""),
        "sanitized event still identifiable, got: {}",
        content
    );
}

// ---------------------------------------------------------------------------
// UserPath newtype — raw() escape hatch exists and confinement unwraps only
// ---------------------------------------------------------------------------

#[test]
fn userpath_raw_escape_hatch_is_documented_surface() {
    // The newtype is deliberately NOT adopted as a tool-input type across the
    // board (all tools still take String); `raw()` is the escape hatch. The
    // confinement guarantee lives at the chokepoint (confine/confine_strict),
    // which every write path funnels through.
    let up = wm_core::shared::models::UserPath::new("specs/x.md");
    let confined = up
        .confine_strict_under(Path::new(".wm/wiki"))
        .expect("valid path confines");
    assert!(confined.starts_with(".wm/wiki"));
    assert_eq!(up.raw(), "specs/x.md");
}
