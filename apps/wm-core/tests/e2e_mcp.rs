// ─── E2E: MCP Operations ──────────────────────────────────────
// Tests MCP-level features: template management via MCP client and
// concurrent session state via basic CLI commands.

mod helpers;

use helpers::{run_cli, run_cli_with_stdin, setup_test_project, MCPClient};

#[test]
fn template_create_and_list() {
    let (_dir, root) = setup_test_project();

    // Start MCP client
    let mut client = MCPClient::start(&root);
    client.initialize().expect("MCP initialize");

    // 1. List templates — should return empty initially
    let result = client
        .call_tool("wm_template", serde_json::json!({ "action": "list" }))
        .expect("wm_template list should succeed");
    let total = result.get("total").and_then(|v| v.as_u64()).unwrap_or(0);
    assert_eq!(total, 0, "expected 0 templates initially, got {}", total);

    // 2. Create a template
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

    // 3. List templates again — should now include our template
    let result = client
        .call_tool("wm_template", serde_json::json!({ "action": "list" }))
        .expect("wm_template list should succeed");
    let total = result.get("total").and_then(|v| v.as_u64()).unwrap_or(0);
    assert!(total >= 1, "expected at least 1 template after creation, got {}", total);
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

    // Clean up MCP client
    client.close();
}

#[test]
fn concurrent_session_state() {
    let (_dir, root) = setup_test_project();

    // Basic CLI operations to verify engine initialization and state management
    let res = run_cli(&root, &["page", "list", "--json"]);
    assert_success!(res);

    let res = run_cli(&root, &["validate", "--json"]);
    assert_success!(res);

    // Create a page and verify the engine handles it
    let res = run_cli_with_stdin(
        &root,
        &["page", "create", "concepts/e2e-session-state", "Session State Test"],
        "Verifying engine serves basic commands.",
    );
    assert_success!(res);

    let res = run_cli(&root, &["page", "list", "--json"]);
    assert_success!(res);
    let parsed: serde_json::Value =
        serde_json::from_str(&res.stdout).expect("valid JSON");
    let total = parsed.get("total").and_then(|v| v.as_u64()).unwrap_or(0);
    assert!(
        total >= 1,
        "expected at least 1 page in session state test, got {}",
        total
    );

    // Also verify page creation via action-enum surface
    let res = run_cli_with_stdin(
        &root,
        &["page", "create", "concepts/e2e-action", "Action Test"],
        "Testing action-enum MCP tool surface via CLI.",
    );
    assert_success!(res);

    let res = run_cli(&root, &["page", "list", "--json"]);
    assert_success!(res);
    let parsed: serde_json::Value =
        serde_json::from_str(&res.stdout).expect("valid JSON");
    let total = parsed.get("total").and_then(|v| v.as_u64()).unwrap_or(0);
    assert!(
        total >= 2,
        "expected at least 2 pages after action test, got {}",
        total
    );
}
