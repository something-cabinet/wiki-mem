#[path = "helpers/cli.rs"]
mod helpers;
use helpers::{run_cli, run_cli_with_stdin};

#[path = "helpers/mcp_basic.rs"]
mod mcp;
use mcp::MCPClient;

#[path = "helpers/setup.rs"]
mod setup;
use setup::setup_test_project;

#[path = "helpers/macros.rs"]
mod _macros;

#[test]
fn template_create_and_list() {
    let (_dir, root) = setup_test_project();

    let mut client = MCPClient::start(&root);
    client.initialize().expect("MCP initialize");

    let result = client
        .call_tool("wm_template", serde_json::json!({ "action": "list" }))
        .expect("wm_template list should succeed");
    let total = result.get("total").and_then(|v| v.as_u64()).unwrap_or(0);
    assert_eq!(total, 0, "expected 0 templates initially, got {}", total);

    let result = client
        .call_tool(
            "wm_template",
            serde_json::json!({
                "action": "create",
                "name": "e2e-test-template",
                "description": "E2E test template",
                "content": "Hello {{name}}! This is an E2E test.",
            }),
        )
        .expect("wm_template create should succeed");
    assert_eq!(
        result.get("status").and_then(|v| v.as_str()),
        Some("created"),
        "template should be created"
    );

    let result = client
        .call_tool("wm_template", serde_json::json!({ "action": "list" }))
        .expect("wm_template list should succeed");
    let total = result.get("total").and_then(|v| v.as_u64()).unwrap_or(0);
    assert!(
        total >= 1,
        "expected at least 1 template after creation, got {}",
        total
    );
    let empty_templates = vec![];
    let templates = result
        .get("templates")
        .and_then(|v| v.as_array())
        .unwrap_or(&empty_templates);
    let has_test_template = templates.iter().any(|t| {
        t.get("name")
            .and_then(|v| v.as_str())
            .map(|s| s == "e2e-test-template")
            .unwrap_or(false)
    });
    assert!(
        has_test_template,
        "e2e-test-template should appear in template list"
    );

    client.close();
}

#[test]
fn concurrent_session_state() {
    let (_dir, root) = setup_test_project();

    let res = run_cli(&root, &["page", "list", "--json"]);
    assert_success!(res);

    let res = run_cli(&root, &["validate", "--json"]);
    assert_success!(res);

    let res = run_cli_with_stdin(
        &root,
        &[
            "page",
            "create",
            "concepts/e2e-session-state",
            "Session State Test",
        ],
        "Verifying engine serves basic commands.",
    );
    assert_success!(res);

    let res = run_cli(&root, &["page", "list", "--json"]);
    assert_success!(res);
    let parsed: serde_json::Value = serde_json::from_str(&res.stdout).expect("valid JSON");
    let total = parsed.get("total").and_then(|v| v.as_u64()).unwrap_or(0);
    assert!(
        total >= 1,
        "expected at least 1 page in session state test, got {}",
        total
    );

    let res = run_cli_with_stdin(
        &root,
        &["page", "create", "concepts/e2e-action", "Action Test"],
        "Testing action-enum MCP tool surface via CLI.",
    );
    assert_success!(res);

    let res = run_cli(&root, &["page", "list", "--json"]);
    assert_success!(res);
    let parsed: serde_json::Value = serde_json::from_str(&res.stdout).expect("valid JSON");
    let total = parsed.get("total").and_then(|v| v.as_u64()).unwrap_or(0);
    assert!(
        total >= 2,
        "expected at least 2 pages after action test, got {}",
        total
    );
}

/// Regression for wiki-tool-reliability bugs B1/B2/B5: a page that exists on
/// disk but is NOT in the in-memory graph index (stale index, e.g. created
/// externally or before an index rebuild) must still be updatable via
/// wm_page.update and wm_task.update — previously these returned phantom
/// "page not found" while wm_page.get worked.
#[test]
fn update_works_with_stale_graph_index() {
    let (_dir, root) = setup_test_project();

    let mut client = MCPClient::start(&root);
    client.initialize().expect("MCP initialize");

    // Write page files directly to disk, bypassing the graph index so the
    // in-memory index is stale relative to what's on disk.
    let task_file = root.join(".wm").join("wiki").join("tasks").join("stale-index-task.md");
    let concept_file = root
        .join(".wm")
        .join("wiki")
        .join("concepts")
        .join("stale-index-concept.md");
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

    // wm_page.update on a page absent from the graph index must succeed.
    let res = client
        .call_tool(
            "wm_page",
            serde_json::json!({
                "action": "update",
                "id": "wiki:concepts:stale-index-concept",
                "status": "active",
                "tags": ["stale", "index", "updated"],
            }),
        )
        .expect("wm_page.update on stale-index page should succeed");
    assert_eq!(
        res.get("status").and_then(|v| v.as_str()),
        Some("updated"),
        "wm_page.update should report updated, got {}",
        res
    );

    // The disk file must reflect the update (tags preserved, not discarded).
    let concept_content =
        std::fs::read_to_string(&concept_file).expect("read updated concept file");
    assert!(
        concept_content.contains("status: active"),
        "concept status should be active on disk"
    );
    assert!(
        concept_content.contains("updated"),
        "concept tags should include 'updated' on disk"
    );

    // wm_task.update on a task absent from the graph index must succeed too.
    let res = client
        .call_tool(
            "wm_task",
            serde_json::json!({
                "action": "update",
                "id": "wiki:tasks:stale-index-task",
                "status": "in-progress",
                "title": "Stale Index Task (updated)",
            }),
        )
        .expect("wm_task.update on stale-index task should succeed");
    assert_eq!(
        res.get("status").and_then(|v| v.as_str()),
        Some("updated"),
        "wm_task.update should report updated, got {}",
        res
    );

    let task_content = std::fs::read_to_string(&task_file).expect("read updated task file");
    assert!(
        task_content.contains("status: in-progress"),
        "task status should be in-progress on disk"
    );
    assert!(
        task_content.contains("Stale Index Task (updated)"),
        "task title should be updated on disk"
    );

    client.close();
}
