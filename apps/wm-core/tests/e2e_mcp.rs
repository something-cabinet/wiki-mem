//! MCP end-to-end behavior contracts (in-process): template lifecycle and the
//! stale-graph-index regression. The transport seam itself lives in
//! mcp_test.rs (stdio); these dispatch through the same registry in-process.

#[path = "helpers/inproc.rs"]
mod inproc;
use inproc::{call, call_err, call_ok, setup_in_process};

use serde_json::json;

async fn rebuild(registry: &wm_core::mcp::transport::ToolRegistry) -> serde_json::Value {
    call_ok(
        registry,
        "wm_index_rebuild",
        json!({ "skip_embed": true }),
    )
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn template_create_and_list() {
    let ((_dir, _root, _engine, registry), _cwd) = setup_in_process().await;
    let initial = call_ok(&registry, "wm_template", json!({ "action": "list" })).await;
    assert_eq!(initial.get("total").and_then(|v| v.as_u64()), Some(0));

    let out = call_ok(
        &registry,
        "wm_template",
        json!({
            "action": "create",
            "name": "e2e-test-template",
            "description": "E2E test template",
            "content": "Hello {{name}}! This is an E2E test.",
        }),
    )
    .await;
    assert_eq!(out.get("status").and_then(|v| v.as_str()), Some("created"));

    let listed = call_ok(&registry, "wm_template", json!({ "action": "list" })).await;
    assert_eq!(listed.get("total").and_then(|v| v.as_u64()), Some(1));
    let templates = listed.get("templates").and_then(|v| v.as_array()).expect("templates");
    assert!(
        templates.iter().any(|t| t.get("name").and_then(|v| v.as_str()) == Some("e2e-test-template")),
        "created template must appear in list"
    );
}

/// Regression (wiki-tool-reliability B1/B2/B5): a page that exists on disk but
/// is NOT in the in-memory graph index (stale index) must still be updatable
/// via wm_page.update and wm_task.update — previously these returned phantom
/// "page not found" while wm_page.get worked.
#[tokio::test(flavor = "multi_thread")]
async fn update_works_with_stale_graph_index() {
    let ((_dir, root, _engine, registry), _cwd) = setup_in_process().await;

    let task_file = root.join(".wm/wiki/tasks/stale-index-task.md");
    let concept_file = root.join(".wm/wiki/concepts/stale-index-concept.md");
    std::fs::write(
        &task_file,
        "---\ntitle: Stale Index Task\ntype: task\nid: wiki:tasks:stale-index-task\nstatus: todo\ntags: [stale, index]\n---\n\nBody content.\n",
    )
    .expect("write task file");
    std::fs::write(
        &concept_file,
        "---\ntitle: Stale Index Concept\ntype: concept\nid: wiki:concepts:stale-index-concept\nstatus: draft\ntags: [stale, index]\n---\n\nConcept body.\n",
    )
    .expect("write concept file");

    let res = call_ok(
        &registry,
        "wm_page",
        json!({
            "action": "update",
            "id": "wiki:concepts:stale-index-concept",
            "status": "reviewed",
            "tags": ["stale", "index", "updated"],
        }),
    )
    .await;
    assert_eq!(res.get("status").and_then(|v| v.as_str()), Some("updated"), "got {res}");

    let concept_content = std::fs::read_to_string(&concept_file).expect("read concept file");
    assert!(concept_content.contains("status: reviewed"), "got: {concept_content}");
    assert!(concept_content.contains("updated"), "tags must include 'updated'");

    let res = call_ok(
        &registry,
        "wm_task",
        json!({
            "action": "update",
            "id": "wiki:tasks:stale-index-task",
            "status": "in-progress",
            "title": "Stale Index Task (updated)",
        }),
    )
    .await;
    assert_eq!(res.get("status").and_then(|v| v.as_str()), Some("updated"), "got {res}");

    let task_content = std::fs::read_to_string(&task_file).expect("read task file");
    assert!(task_content.contains("status: in-progress"), "got: {task_content}");
    assert!(task_content.contains("Stale Index Task (updated)"));

    // The stale-index fix must not silently accept truly missing pages.
    let err = call_err(
        &registry,
        "wm_page",
        json!({ "action": "update", "id": "wiki:concepts:no-such-page", "title": "Ghost" }),
    )
    .await;
    assert!(
        err.message.contains("not found"),
        "updating a missing page must error, got: {}",
        err.message
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn concurrent_session_state() {
    let ((_dir, _root, _engine, registry), _cwd) = setup_in_process().await;
    let out = call(&registry, "wm_page", json!({ "action": "list" }))
        .await
        .expect("page list");
    assert_eq!(out.get("total").and_then(|v| v.as_u64()), Some(0));

    call_ok(
        &registry,
        "wm_page",
        json!({ "action": "create", "path": "concepts/e2e-session-state", "title": "Session State Test", "content": "Body." }),
    )
    .await;
    let out = call_ok(&registry, "wm_page", json!({ "action": "list" })).await;
    assert_eq!(out.get("total").and_then(|v| v.as_u64()), Some(1));

    call_ok(
        &registry,
        "wm_page",
        json!({ "action": "create", "path": "concepts/e2e-action", "title": "Action Test", "content": "Body." }),
    )
    .await;
    let out = call_ok(&registry, "wm_page", json!({ "action": "list" })).await;
    assert_eq!(out.get("total").and_then(|v| v.as_u64()), Some(2));

    rebuild(&registry).await;
    let out = call_ok(&registry, "wm_search.query", json!({ "q": "Action Test", "limit": 5 })).await;
    assert!(out.get("results").and_then(|v| v.as_array()).is_some());
}
