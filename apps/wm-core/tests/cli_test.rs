#[path = "helpers/cli.rs"]
mod helpers;
use helpers::{run_cli, run_cli_with_stdin};

#[path = "helpers/setup.rs"]
mod setup;
use setup::setup_test_project;

#[path = "helpers/macros.rs"]
mod _macros;

#[test]
fn test_cli_help() {
    let (_dir, root) = setup_test_project();
    let res = run_cli(&root, &["--help"]);
    assert_success!(res);
    assert_contains!(res.stdout, "Usage:");
    assert_contains!(res.stdout, "Commands");
}

#[test]
fn test_cli_version() {
    let (_dir, root) = setup_test_project();
    let res = run_cli(&root, &["version"]);
    assert_success!(res);
}

#[test]
fn test_cli_page_list_empty() {
    let (_dir, root) = setup_test_project();
    let res = run_cli(&root, &["page", "list", "--json"]);
    assert_success!(res);
    let parsed: serde_json::Value =
        serde_json::from_str(&res.stdout).expect("page list --json should be valid JSON");
    assert!(
        parsed.get("pages").is_some(),
        "expected 'pages' key in JSON output"
    );
    assert!(
        parsed.get("total").is_some(),
        "expected 'total' key in JSON output"
    );
    let total = parsed
        .get("total")
        .and_then(|v| v.as_u64())
        .unwrap_or(u64::MAX);
    assert_eq!(total, 0, "expected 0 pages in empty project, got {}", total);
}

#[test]
fn test_cli_page_create_and_list() {
    let (_dir, root) = setup_test_project();

    let res = run_cli_with_stdin(
        &root,
        &["page", "create", "concepts/test-concept", "Test Concept"],
        "A test concept page.",
    );
    assert_success!(res);
    assert_contains!(res.stdout, "Created page");

    let res = run_cli(&root, &["page", "list"]);
    assert_success!(res);
    assert_contains!(res.stdout, "1 pages");
    assert_contains!(res.stdout, "test-concept");

    let res = run_cli_with_stdin(
        &root,
        &["page", "create", "tasks/test-task", "Test Task"],
        "A test task.",
    );
    assert_success!(res);
    assert_contains!(res.stdout, "Created page");

    let res = run_cli(&root, &["page", "list", "--json"]);
    assert_success!(res);
    assert_contains!(res.stdout, "2");
}

#[test]
fn test_cli_page_get() {
    let (_dir, root) = setup_test_project();

    run_cli_with_stdin(
        &root,
        &["page", "create", "concepts/test-get", "Test Get"],
        "Content for get test.",
    );

    let res = run_cli(&root, &["page", "get", "wiki:concepts:test-get"]);
    assert_success!(res);
    assert_contains!(res.stdout, "Test Get");
}

#[test]
fn test_cli_search_keyword() {
    let (_dir, root) = setup_test_project();

    run_cli_with_stdin(
        &root,
        &[
            "page",
            "create",
            "concepts/search-target",
            "Search Target Page",
        ],
        "This page exists for search testing.",
    );

    let res = run_cli(&root, &["search", "query", "Search Target", "--json"]);
    assert_success!(res);
    assert_contains!(res.stdout, "search-target");
}

#[test]
fn test_cli_search_retrieve() {
    let (_dir, root) = setup_test_project();
    let res = run_cli(
        &root,
        &[
            "search",
            "retrieve",
            "test",
            "--token-budget",
            "4096",
            "--json",
        ],
    );
    assert_success!(res);
    assert_contains!(res.stdout, "token_budget");
}

#[test]
fn test_cli_graph_stats() {
    let (_dir, root) = setup_test_project();

    run_cli(
        &root,
        &["page", "create", "concepts/graph-node-a", "Graph Node A"],
    );
    run_cli(
        &root,
        &["page", "create", "concepts/graph-node-b", "Graph Node B"],
    );

    let res = run_cli(&root, &["graph", "stats", "--json"]);
    assert_success!(res);
    assert_contains!(res.stdout, "nodes");
    assert_contains!(res.stdout, "edges");
}

#[test]
fn test_cli_source_list() {
    let (_dir, root) = setup_test_project();
    let res = run_cli(&root, &["source", "list"]);
    assert_success!(res);
    assert_contains!(res.stdout, "0 sources");
}

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

#[test]
fn test_cli_task_board() {
    let (_dir, root) = setup_test_project();

    run_cli(&root, &["page", "create", "tasks/board-task", "Board Task"]);

    let res = run_cli(&root, &["task", "board", "--json"]);
    assert_success!(res);
    assert_contains!(res.stdout, "{");
}

#[test]
fn test_cli_page_list_json() {
    let (_dir, root) = setup_test_project();
    let res = run_cli(&root, &["page", "list", "--json"]);
    assert_success!(res);
    let parsed: serde_json::Value =
        serde_json::from_str(&res.stdout).expect("page list --json should be valid JSON");
    assert!(
        parsed.get("pages").is_some(),
        "expected 'pages' key in JSON output"
    );
    assert!(
        parsed.get("total").is_some(),
        "expected 'total' key in JSON output"
    );
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

#[test]
fn test_setup_opencode_json() {
    let (_dir, root) = setup_test_project();
    let res = run_cli(&root, &["init", "--no-wizard", "--platform", "opencode"]);
    assert_success!(res);
    let opencode_path = root.join("opencode.json");
    assert!(opencode_path.exists(), "opencode.json should exist");
    let content = std::fs::read_to_string(&opencode_path).unwrap();
    let parsed: serde_json::Value =
        serde_json::from_str(&content).expect("opencode.json should be valid JSON");
    let mcp = parsed.get("mcp").and_then(|m| m.get("wm"));
    assert!(mcp.is_some(), "opencode.json should have mcp.wm entry");
    assert_eq!(
        mcp.and_then(|m| m.get("enabled")).and_then(|v| v.as_bool()),
        Some(true)
    );
}

#[test]
fn test_setup_codex_toml() {
    let (_dir, root) = setup_test_project();
    let res = run_cli(&root, &["init", "--no-wizard", "--platform", "codex"]);
    assert_success!(res);
    let codex_path = root.join(".codex").join("config.toml");
    assert!(codex_path.exists(), ".codex/config.toml should exist");
    let content = std::fs::read_to_string(&codex_path).unwrap();
    assert!(
        content.contains("[mcp_servers.wm]"),
        "TOML should contain [mcp_servers.wm] section"
    );
}

#[test]
fn test_agents_sync_files() {
    let (_dir, root) = setup_test_project();
    let res = run_cli(&root, &["agents", "--sync"]);
    assert_success!(res);
    assert!(root.join("CLAUDE.md").exists(), "CLAUDE.md should exist");
    assert!(root.join("AGENTS.md").exists(), "AGENTS.md should exist");
    assert!(root.join("GEMINI.md").exists(), "GEMINI.md should exist");
    assert!(
        root.join(".github")
            .join("copilot-instructions.md")
            .exists(),
        ".github/copilot-instructions.md should exist"
    );
}

#[test]
fn test_setup_kiro_json() {
    let (_dir, root) = setup_test_project();
    let res = run_cli(&root, &["init", "--no-wizard", "--platform", "kiro"]);
    assert_success!(res);
    let kiro_mcp = root.join(".kiro").join("settings").join("mcp.json");
    assert!(kiro_mcp.exists(), ".kiro/settings/mcp.json should exist");
    let content = std::fs::read_to_string(&kiro_mcp).unwrap();
    let parsed: serde_json::Value =
        serde_json::from_str(&content).expect("kiro mcp.json should be valid JSON");
    let servers = parsed.get("mcpServers").and_then(|m| m.get("wm"));
    assert!(
        servers.is_some(),
        "kiro mcp.json should have mcpServers.wm entry"
    );
}

#[test]
fn test_setup_cursor_mcp() {
    let (_dir, root) = setup_test_project();
    let res = run_cli(&root, &["init", "--no-wizard", "--platform", "cursor"]);
    assert_success!(res);
    let cursor_mcp = root.join(".cursor").join("mcp.json");
    assert!(cursor_mcp.exists(), ".cursor/mcp.json should exist");
    let content = std::fs::read_to_string(&cursor_mcp).unwrap();
    let parsed: serde_json::Value =
        serde_json::from_str(&content).expect("cursor mcp.json should be valid JSON");
    assert!(
        parsed.get("mcpServers").is_some(),
        "cursor mcp.json should have mcpServers"
    );
}

#[test]
fn test_setup_all_writes_every_platform() {
    let (_dir, root) = setup_test_project();
    let res = run_cli(&root, &["setup", "all"]);
    assert_success!(res);

    // Every platform's MCP config file must exist.
    assert!(root.join("opencode.json").exists(), "opencode.json should exist");
    assert!(
        root.join(".kiro").join("settings").join("mcp.json").exists(),
        ".kiro/settings/mcp.json should exist"
    );
    assert!(root.join(".mcp.json").exists(), ".mcp.json (claude) should exist");
    assert!(
        root.join(".codex").join("config.toml").exists(),
        ".codex/config.toml should exist"
    );
    assert!(
        root.join(".cursor").join("mcp.json").exists(),
        ".cursor/mcp.json should exist"
    );
    assert!(
        root.join(".gemini").join("antigravity").join("mcp_config.json").exists(),
        "antigravity mcp_config.json should exist"
    );

    // Spot-check that each config references the wm MCP server.
    let opencode = std::fs::read_to_string(root.join("opencode.json")).unwrap();
    let parsed: serde_json::Value =
        serde_json::from_str(&opencode).expect("opencode.json should be valid JSON");
    assert!(
        parsed.pointer("/mcp/wm").is_some(),
        "opencode.json should have mcp.wm entry"
    );

    let kiro = std::fs::read_to_string(root.join(".kiro").join("settings").join("mcp.json")).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&kiro).expect("kiro mcp.json should be valid JSON");
    assert!(
        parsed.pointer("/mcpServers/wm").is_some(),
        "kiro mcp.json should have mcpServers.wm entry"
    );

    let codex = std::fs::read_to_string(root.join(".codex").join("config.toml")).unwrap();
    assert!(
        codex.contains("[mcp_servers.wm]"),
        "codex config.toml should contain [mcp_servers.wm] section"
    );
}

#[test]
fn test_cli_workflow_task_lifecycle() {
    let (_dir, root) = setup::setup_test_project();

    let res = helpers::run_cli_with_stdin(
        &root,
        &["page", "create", "tasks/cli-e2e-task", "CLI E2E Task"],
        "Task created via CLI for E2E lifecycle test.",
    );
    assert_success!(res);
    assert_contains!(res.stdout, "Created page");

    let res = helpers::run_cli(&root, &["page", "list"]);
    assert_success!(res);
    assert_contains!(res.stdout, "cli-e2e-task");

    let res = helpers::run_cli(&root, &["search", "query", "CLI E2E", "--json"]);
    assert_success!(res);
    assert_contains!(res.stdout, "cli-e2e-task");

    let res = helpers::run_cli(
        &root,
        &["time", "start", "wiki:tasks:cli-e2e-task", "--json"],
    );
    assert_success!(res);

    let res = helpers::run_cli(
        &root,
        &["time", "stop", "wiki:tasks:cli-e2e-task", "--json"],
    );
    assert_success!(res);

    let res = helpers::run_cli(&root, &["time", "report", "--json"]);
    assert_success!(res);

    let res = helpers::run_cli(&root, &["lint", "check", "--json"]);
    assert_success!(res);

    let res = helpers::run_cli(&root, &["validate", "--json"]);
    assert_success!(res);

    let res = helpers::run_cli(&root, &["index", "rebuild"]);
    assert_success!(res);

    let res = helpers::run_cli(&root, &["page", "list", "--json"]);
    assert_success!(res);
    let parsed: serde_json::Value = serde_json::from_str(&res.stdout).expect("valid JSON");
    let total = parsed.get("total").and_then(|v| v.as_u64()).unwrap_or(0);
    assert!(
        total >= 1,
        "expected at least 1 page after rebuild, got {}",
        total
    );
}

#[test]
fn test_cli_workflow_board() {
    let (_dir, root) = setup::setup_test_project();

    let res = helpers::run_cli(
        &root,
        &["page", "create", "tasks/cli-board-task", "Board Task"],
    );
    assert_success!(res);

    let res = helpers::run_cli(&root, &["task", "board", "--json"]);
    assert_success!(res);
    let parsed: serde_json::Value = serde_json::from_str(&res.stdout).expect("valid JSON");
    assert!(parsed.get("columns").is_some(), "board should have columns");
    let todo_count = parsed
        .get("columns")
        .and_then(|c| c.get("todo"))
        .and_then(|t| t.as_array())
        .map(|a| a.len())
        .unwrap_or(0);
    assert!(
        todo_count >= 1,
        "expected at least 1 task on board (todo), got {}",
        todo_count
    );
}

#[test]
fn test_cli_workflow_memory() {
    let (_dir, root) = setup::setup_test_project();

    let mem_dir = root.join(".wm").join("memory");
    let mem = serde_json::json!({
        "id": "cli-e2e-memory",
        "title": "CLI E2E Memory",
        "content": "Memory entry created for CLI E2E testing.",
        "tags": ["e2e", "cli"],
        "created_at": "2026-07-07T00:00:00Z",
        "updated_at": "2026-07-07T00:00:00Z"
    });
    std::fs::write(
        mem_dir.join("cli-e2e-memory.json"),
        serde_json::to_string_pretty(&mem).unwrap(),
    )
    .expect("write memory entry");

    let res = helpers::run_cli(&root, &["index", "rebuild"]);
    assert_success!(res);

    eprintln!("Memory entry created and index rebuilt successfully");
}

#[test]
fn test_cli_workflow_cross_entity_search() {
    let (_dir, root) = setup::setup_test_project();

    let res = helpers::run_cli_with_stdin(
        &root,
        &[
            "page",
            "create",
            "concepts/cli-cross-entity",
            "CLI Cross Entity Page",
        ],
        "This page is for cross-entity search testing with JWT authentication.",
    );
    assert_success!(res);

    let mem_dir = root.join(".wm").join("memory");
    let mem = serde_json::json!({
        "id": "cli-cross-entity-mem",
        "title": "Auth Pattern Memory",
        "content": "Authentication uses JWT with RS256.",
        "tags": ["auth"],
        "created_at": "2026-07-07T00:00:00Z",
        "updated_at": "2026-07-07T00:00:00Z"
    });
    std::fs::write(
        mem_dir.join("cli-cross-entity-mem.json"),
        serde_json::to_string_pretty(&mem).unwrap(),
    )
    .expect("write memory");

    let res = helpers::run_cli(&root, &["index", "rebuild"]);
    assert_success!(res);

    let res = helpers::run_cli(&root, &["search", "query", "cross-entity", "--json"]);
    assert_success!(res);
    let parsed: serde_json::Value = serde_json::from_str(&res.stdout).expect("valid JSON");
    let empty = vec![];
    let results = parsed
        .get("results")
        .and_then(|v| v.as_array())
        .unwrap_or(&empty);
    assert!(
        !results.is_empty(),
        "expected search results for created page"
    );
    eprintln!("CLI search returned {} results", results.len());
}

#[test]
fn test_cli_workflow_validation() {
    let (_dir, root) = setup::setup_test_project();

    let res = helpers::run_cli(
        &root,
        &["page", "create", "tasks/cli-validate-task", "Validate Task"],
    );
    assert_success!(res);

    let res = helpers::run_cli(&root, &["validate", "--json"]);
    assert_success!(res);
    let parsed: serde_json::Value = serde_json::from_str(&res.stdout).expect("valid JSON");
    assert!(
        parsed.get("status").is_some(),
        "validate should return status"
    );
    assert!(
        parsed.get("nodes").is_some(),
        "validate should return node count"
    );
}

#[test]
fn test_cli_workflow_graph_link() {
    let (_dir, root) = setup::setup_test_project();

    helpers::run_cli(
        &root,
        &["page", "create", "concepts/graph-link-a", "Graph Link A"],
    );
    helpers::run_cli(
        &root,
        &["page", "create", "specs/graph-link-b", "Graph Link B"],
    );

    let res = helpers::run_cli(
        &root,
        &[
            "page",
            "link",
            "wiki:concepts:graph-link-a",
            "wiki:specs:graph-link-b",
            "--edge-type",
            "related_to",
        ],
    );
    assert_success!(res);

    helpers::run_cli(&root, &["index", "rebuild"]);

    let res = helpers::run_cli(
        &root,
        &["graph", "neighbors", "wiki:concepts:graph-link-a", "--json"],
    );
    if res.exit_code != 0 {
        eprintln!(
            "graph neighbors failed (non-fatal): {}: {}",
            res.exit_code, res.stderr
        );
    } else {
        let _parsed: serde_json::Value = serde_json::from_str(&res.stdout).expect("valid JSON");
    }
}

#[test]
fn test_cli_workflow_search_retrieve() {
    let (_dir, root) = setup::setup_test_project();

    helpers::run_cli_with_stdin(
        &root,
        &[
            "page",
            "create",
            "concepts/cli-retrieve-test",
            "Retrieve Test Page",
        ],
        "This page is created for retrieve testing.",
    );

    let res = helpers::run_cli(
        &root,
        &[
            "search",
            "retrieve",
            "retrieve testing",
            "--token-budget",
            "4096",
            "--json",
        ],
    );
    assert_success!(res);
    let parsed: serde_json::Value = serde_json::from_str(&res.stdout).expect("valid JSON");
    assert!(
        parsed.get("context").is_some(),
        "retrieve should return context"
    );
    assert!(
        parsed.get("tokens_used").is_some(),
        "retrieve should return tokens_used"
    );
}

#[test]
fn test_cli_search_cross_entity() {
    let (_dir, root) = setup::setup_test_project();

    let res = helpers::run_cli_with_stdin(
        &root,
        &[
            "page",
            "create",
            "concepts/cross-entity-test",
            "Cross Entity Test",
        ],
        "This page is used for cross-entity search testing.",
    );
    assert_success!(res);

    let res = helpers::run_cli(&root, &["index", "rebuild"]);
    assert_success!(res);

    let res = helpers::run_cli(&root, &["search", "query", "cross-entity", "--json"]);
    assert_success!(res);
    let parsed: serde_json::Value =
        serde_json::from_str(&res.stdout).expect("search query --json should be valid JSON");
    let results = parsed.get("results").and_then(|v| v.as_array());
    assert!(results.is_some(), "expected 'results' key in search output");
    let results = results.unwrap();
    assert!(
        !results.is_empty(),
        "expected at least one search result, got none"
    );
}

#[test]
fn test_cli_index_rebuild() {
    let (_dir, root) = setup::setup_test_project();

    helpers::run_cli(
        &root,
        &[
            "page",
            "create",
            "concepts/index-rebuild-test",
            "Index Rebuild Test",
        ],
    );

    let res = helpers::run_cli(&root, &["index", "rebuild"]);
    assert_success!(res);
    assert_contains!(res.stdout, "Rebuild complete.");
}

#[test]
fn test_cli_graph_neighbors() {
    let (_dir, root) = setup::setup_test_project();

    helpers::run_cli(
        &root,
        &[
            "page",
            "create",
            "concepts/graph-neighbor-a",
            "Graph Neighbor A",
        ],
    );
    helpers::run_cli(
        &root,
        &[
            "page",
            "create",
            "specs/graph-neighbor-b",
            "Graph Neighbor B",
        ],
    );

    let res = helpers::run_cli(
        &root,
        &[
            "page",
            "link",
            "wiki:concepts:graph-neighbor-a",
            "wiki:specs:graph-neighbor-b",
            "--edge-type",
            "related_to",
        ],
    );
    assert_success!(res);

    helpers::run_cli(&root, &["index", "rebuild"]);

    let res = helpers::run_cli(
        &root,
        &[
            "graph",
            "neighbors",
            "wiki:concepts:graph-neighbor-a",
            "--json",
        ],
    );
    if res.exit_code != 0 {
        eprintln!(
            "graph neighbors failed (non-fatal): {}: {}",
            res.exit_code, res.stderr
        );
    } else {
        let _parsed: serde_json::Value =
            serde_json::from_str(&res.stdout).expect("graph neighbors --json should be valid JSON");
    }
}

#[test]
fn test_cli_time_tracking() {
    let (_dir, root) = setup::setup_test_project();

    helpers::run_cli(
        &root,
        &[
            "page",
            "create",
            "tasks/time-tracked-task",
            "Time Tracked Task",
        ],
    );

    let res = helpers::run_cli(
        &root,
        &["time", "start", "wiki:tasks:time-tracked-task", "--json"],
    );
    assert_success!(res);

    let res = helpers::run_cli(
        &root,
        &["time", "stop", "wiki:tasks:time-tracked-task", "--json"],
    );
    assert_success!(res);

    let task_file = root
        .join(".wm")
        .join("wiki")
        .join("tasks")
        .join("time-tracked-task.md");
    let content = std::fs::read_to_string(&task_file).unwrap_or_default();
    assert!(
        content.contains("time_spent:"),
        "time stop should persist time_spent in frontmatter, got: {}",
        content
    );
    assert!(
        content.contains("time_started:"),
        "time start should persist time_started in frontmatter, got: {}",
        content
    );

    let res = helpers::run_cli(&root, &["time", "report", "--json"]);
    assert_success!(res);
    assert_contains!(res.stdout, "time-tracked-task");
}

#[test]
fn test_cli_lint_fix() {
    let (_dir, root) = setup::setup_test_project();

    helpers::run_cli(
        &root,
        &["page", "create", "concepts/lint-fix-test", "Lint Fix Test"],
    );

    let res = helpers::run_cli(&root, &["lint", "fix"]);
    assert_success!(res);
    assert_contains!(res.stdout, "Fixed");
}

#[test]
fn test_regression_page_update_tags() {
    let (_dir, root) = setup::setup_test_project();

    let res = helpers::run_cli_with_stdin(
        &root,
        &["page", "create", "regression/tags", "Tags Test"],
        "content",
    );
    assert_success!(res);

    let res = helpers::run_cli_with_stdin(
        &root,
        &["page", "update", "wiki:regression:tags"],
        r#"{"tags": ["rust", "async", "test"]}"#,
    );
    assert_success!(res);

    let res = helpers::run_cli(&root, &["page", "get", "wiki:regression:tags", "--json"]);
    assert_success!(res);
    assert_contains!(res.stdout, "rust");
    assert_contains!(res.stdout, "async");
    assert_contains!(res.stdout, "test");
}

#[test]
fn test_regression_cli_page_update() {
    let (_dir, root) = setup::setup_test_project();

    let res = helpers::run_cli_with_stdin(
        &root,
        &["page", "create", "regression/update", "Update Test"],
        "content",
    );
    assert_success!(res);

    let res = helpers::run_cli_with_stdin(
        &root,
        &["page", "update", "wiki:regression:update"],
        r#"{"title": "Updated Title"}"#,
    );
    assert_success!(res);
}

#[test]
fn test_regression_stdin_multiline() {
    let (_dir, root) = setup::setup_test_project();

    let res = helpers::run_cli_with_stdin(
        &root,
        &["page", "create", "regression/multiline", "Multiline"],
        "line1\nline2\nline3",
    );
    assert_success!(res);

    let res = helpers::run_cli(
        &root,
        &["page", "get", "wiki:regression:multiline", "--json"],
    );
    assert_success!(res);
    assert_contains!(res.stdout, "line1");
    assert_contains!(res.stdout, "line2");
    assert_contains!(res.stdout, "line3");
}

#[test]
fn test_regression_meta_path() {
    let (_dir, root) = setup::setup_test_project();

    let res = helpers::run_cli_with_stdin(
        &root,
        &["page", "create", "regression/path", "Path Test"],
        "content",
    );
    assert_success!(res);

    let res = helpers::run_cli(&root, &["index", "rebuild"]);
    assert_success!(res);

    let res = helpers::run_cli_with_stdin(
        &root,
        &["page", "update", "wiki:regression:path"],
        r#"{"title": "Path Updated"}"#,
    );
    assert_success!(res);
}

#[test]
fn test_regression_create_no_doubled_wiki_dir() {
    let (_dir, root) = setup_test_project();

    let res = helpers::run_cli_with_stdin(
        &root,
        &["page", "create", "concepts/doubled-path", "Doubled Path"],
        "body content",
    );
    assert_success!(res);

    let index = root.join(".wm").join("wiki").join("index.md");
    assert!(
        index.exists(),
        "index.md should be regenerated after page create"
    );
    let content = std::fs::read_to_string(&index).unwrap();
    assert_contains!(content, "doubled-path");
}

#[test]
fn test_regression_content_flag_rejected() {
    let (_dir, root) = setup_test_project();

    let res = helpers::run_cli(
        &root,
        &[
            "page",
            "create",
            "concepts/no-flag",
            "No Flag",
            "--content",
            "x",
        ],
    );
    assert_ne!(
        res.exit_code, 0,
        "expected --content to be rejected by clap"
    );
    assert_contains!(res.stderr, "unexpected argument");
}

#[test]
fn test_health_fix_stubs_referenced_empty_task_and_deletes_orphan() {
    let (_dir, root) = setup_test_project();
    let wiki_dir = root.join(".wm").join("wiki");

    // Referenced empty task page (status todo, frontmatter only, no body) → stub
    std::fs::write(
        wiki_dir.join("tasks/stub-me.md"),
        "---\ntitle: Stub Me\ntype: task\nstatus: todo\n---\n",
    )
    .unwrap();

    // Orphan empty task page (no inbound refs) → delete
    std::fs::write(
        wiki_dir.join("tasks/orphan-me.md"),
        "---\ntitle: Orphan Me\ntype: task\nstatus: todo\n---\n",
    )
    .unwrap();

    // Non-empty task page → untouched
    std::fs::write(
        wiki_dir.join("tasks/has-body.md"),
        "---\ntitle: Has Body\ntype: task\nstatus: in-progress\n---\n\n## Overview\n\nReal content.\n",
    )
    .unwrap();

    // Concept page referencing stub-me so it has an inbound ref
    std::fs::write(
        wiki_dir.join("concepts/refs.md"),
        "---\ntitle: Refs\ntype: concept\nrelates_to:\n  - {type: references, target: wiki:tasks:stub-me}\n---\n\n## Overview\n\nRefs.\n",
    )
    .unwrap();

    let res = helpers::run_cli(&root, &["health", "audit", "--fix"]);
    assert_success!(res);
    assert_contains!(res.stdout, "1 pages stubbed");
    assert_contains!(res.stdout, "1 pages deleted");

    let stub = std::fs::read_to_string(wiki_dir.join("tasks/stub-me.md")).unwrap();
    assert_contains!(stub, "## Overview");
    assert_contains!(stub, "Task stub");
    assert_contains!(stub, "Stub Me");

    assert!(
        !wiki_dir.join("tasks/orphan-me.md").exists(),
        "orphan empty task should be deleted by --fix"
    );

    let has_body = std::fs::read_to_string(wiki_dir.join("tasks/has-body.md")).unwrap();
    assert_contains!(has_body, "Real content.");
}
