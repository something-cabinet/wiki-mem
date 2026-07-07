// ─── CLI E2E Integration Tests ───────────────────────────────
// Following Knowns pattern from tests/e2e_cli_test.go:
//   create project, run CLI commands, verify output

mod helpers;

use helpers::{run_cli, setup_test_project};

// ─── Init / Help ─────────────────────────────────────────────

#[test]
fn test_cli_help() {
    let (_dir, root) = setup_test_project();
    let res = run_cli(&root, &["--help"]);
    assert_success!(res);
    assert_contains!(res.stdout, "Wiki Memory Engine");
    assert_contains!(res.stdout, "Commands");
}

#[test]
fn test_cli_version() {
    let (_dir, root) = setup_test_project();
    let res = run_cli(&root, &["version"]);
    assert_success!(res);
}

// ─── Page Operations ─────────────────────────────────────────

#[test]
fn test_cli_page_list_empty() {
    let (_dir, root) = setup_test_project();
    let res = run_cli(&root, &["page", "list"]);
    assert_success!(res);
    assert_contains!(res.stdout, "0 pages");
}

#[test]
fn test_cli_page_create_and_list() {
    let (_dir, root) = setup_test_project();

    // Create a concept page
    let res = run_cli(&root, &[
        "page", "create", "concepts/test-concept",
        "Test Concept",
        "--content", "A test concept page.",
    ]);
    assert_success!(res);
    assert_contains!(res.stdout, "Created page");

    // List pages
    let res = run_cli(&root, &["page", "list"]);
    assert_success!(res);
    assert_contains!(res.stdout, "1 pages");
    assert_contains!(res.stdout, "test-concept");

    // Create a task page
    let res = run_cli(&root, &[
        "page", "create", "tasks/test-task",
        "Test Task",
        "--content", "A test task.",
    ]);
    assert_success!(res);
    assert_contains!(res.stdout, "Created page");

    // List should show 2
    let res = run_cli(&root, &["page", "list", "--json"]);
    assert_success!(res);
    assert_contains!(res.stdout, "2");
}

#[test]
fn test_cli_page_get() {
    let (_dir, root) = setup_test_project();

    // Create then get
    run_cli(&root, &[
        "page", "create", "concepts/test-get",
        "Test Get",
        "--content", "Content for get test.",
    ]);

    let res = run_cli(&root, &["page", "get", "wiki:concepts:test-get"]);
    assert_success!(res);
    assert_contains!(res.stdout, "Test Get");
}

// ─── Search Operations ───────────────────────────────────────

#[test]
fn test_cli_search_keyword() {
    let (_dir, root) = setup_test_project();

    // Create a page to search for
    run_cli(&root, &[
        "page", "create", "concepts/search-target",
        "Search Target Page",
        "--content", "This page exists for search testing.",
    ]);

    let res = run_cli(&root, &["search", "query", "Search Target", "--json"]);
    assert_success!(res);
    assert_contains!(res.stdout, "search-target");
}

#[test]
fn test_cli_search_retrieve() {
    let (_dir, root) = setup_test_project();
    let res = run_cli(&root, &[
        "search", "retrieve", "test",
        "--token-budget", "4096", "--json",
    ]);
    assert_success!(res);
    assert_contains!(res.stdout, "token_budget");
}

// ─── Graph Operations ────────────────────────────────────────

#[test]
fn test_cli_graph_stats() {
    let (_dir, root) = setup_test_project();

    // Create pages to have nodes in the graph
    run_cli(&root, &[
        "page", "create", "concepts/graph-node-a",
        "Graph Node A",
    ]);
    run_cli(&root, &[
        "page", "create", "concepts/graph-node-b",
        "Graph Node B",
    ]);

    let res = run_cli(&root, &["graph", "stats", "--json"]);
    assert_success!(res);
    assert_contains!(res.stdout, "nodes");
    assert_contains!(res.stdout, "edges");
}

// ─── Source Operations ───────────────────────────────────────

#[test]
fn test_cli_source_list() {
    let (_dir, root) = setup_test_project();
    let res = run_cli(&root, &["source", "list"]);
    assert_success!(res);
    assert_contains!(res.stdout, "0 sources");
}

// ─── Lint & Validate ─────────────────────────────────────────

#[test]
fn test_cli_lint_check() {
    let (_dir, root) = setup_test_project();
    let res = run_cli(&root, &["lint", "check"]);
    assert_success!(res);
    assert_contains!(res.stdout, "Nodes");
}

#[test]
fn test_cli_validate() {
    let (_dir, root) = setup_test_project();
    let res = run_cli(&root, &["validate"]);
    assert_success!(res);
    assert_contains!(res.stdout, "Validation complete");
}

// ─── Task Board ──────────────────────────────────────────────

#[test]
fn test_cli_task_board() {
    let (_dir, root) = setup_test_project();

    // Create a task page
    run_cli(&root, &[
        "page", "create", "tasks/board-task",
        "Board Task",
    ]);

    let res = run_cli(&root, &["task", "board", "--json"]);
    assert_success!(res);
    // Board should at least return valid JSON
    assert_contains!(res.stdout, "{");
}

// ─── JSON output on all commands ────────────────────────────

#[test]
fn test_cli_page_list_json() {
    let (_dir, root) = setup_test_project();
    let res = run_cli(&root, &["page", "list", "--json"]);
    assert_success!(res);
    // Verify valid JSON
    let parsed: serde_json::Value =
        serde_json::from_str(&res.stdout).expect("page list --json should be valid JSON");
    assert!(parsed.get("pages").is_some(), "expected 'pages' key in JSON output");
    assert!(parsed.get("total").is_some(), "expected 'total' key in JSON output");
}

#[test]
fn test_cli_validate_json() {
    let (_dir, root) = setup_test_project();
    let res = run_cli(&root, &["validate", "--json"]);
    assert_success!(res);
    let parsed: serde_json::Value =
        serde_json::from_str(&res.stdout).expect("validate --json should be valid JSON");
    assert!(parsed.get("status").is_some());
}

// ─── Platform Setup Tests ─────────────────────────────────────

#[test]
fn test_setup_opencode_json() {
    let (_dir, root) = setup_test_project();
    let res = run_cli(&root, &["setup", "opencode"]);
    assert_success!(res);
    let opencode_path = root.join("opencode.json");
    assert!(opencode_path.exists(), "opencode.json should exist");
    let content = std::fs::read_to_string(&opencode_path).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&content)
        .expect("opencode.json should be valid JSON");
    let mcp = parsed.get("mcp").and_then(|m| m.get("wm"));
    assert!(mcp.is_some(), "opencode.json should have mcp.wm entry");
    assert_eq!(mcp.and_then(|m| m.get("enabled")).and_then(|v| v.as_bool()), Some(true));
}

#[test]
fn test_setup_codex_toml() {
    let (_dir, root) = setup_test_project();
    let res = run_cli(&root, &["setup", "codex"]);
    assert_success!(res);
    let codex_path = root.join(".codex").join("config.toml");
    assert!(codex_path.exists(), ".codex/config.toml should exist");
    let content = std::fs::read_to_string(&codex_path).unwrap();
    assert!(content.contains("[mcp_servers.wm]"), "TOML should contain [mcp_servers.wm] section");
}

#[test]
fn test_agents_sync_files() {
    let (_dir, root) = setup_test_project();
    let res = run_cli(&root, &["agents", "--sync"]);
    assert_success!(res);
    assert!(root.join("CLAUDE.md").exists(), "CLAUDE.md should exist");
    assert!(root.join("OPENCODE.md").exists(), "OPENCODE.md should exist");
    assert!(root.join("KIRO.md").exists(), "KIRO.md should exist");
    assert!(root.join("GEMINI.md").exists(), "GEMINI.md should exist");
    assert!(root.join(".github").join("copilot-instructions.md").exists(),
        ".github/copilot-instructions.md should exist");
}

#[test]
fn test_setup_kiro_json() {
    let (_dir, root) = setup_test_project();
    let res = run_cli(&root, &["setup", "kiro"]);
    assert_success!(res);
    let kiro_mcp = root.join(".kiro").join("settings").join("mcp.json");
    assert!(kiro_mcp.exists(), ".kiro/settings/mcp.json should exist");
    let content = std::fs::read_to_string(&kiro_mcp).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&content)
        .expect("kiro mcp.json should be valid JSON");
    let servers = parsed.get("mcpServers").and_then(|m| m.get("wm"));
    assert!(servers.is_some(), "kiro mcp.json should have mcpServers.wm entry");
}

#[test]
fn test_setup_cursor_mcp() {
    let (_dir, root) = setup_test_project();
    let res = run_cli(&root, &["setup", "cursor"]);
    assert_success!(res);
    let cursor_mcp = root.join(".cursor").join("mcp.json");
    assert!(cursor_mcp.exists(), ".cursor/mcp.json should exist");
    let content = std::fs::read_to_string(&cursor_mcp).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&content)
        .expect("cursor mcp.json should be valid JSON");
    assert!(parsed.get("mcpServers").is_some(), "cursor mcp.json should have mcpServers");
}
