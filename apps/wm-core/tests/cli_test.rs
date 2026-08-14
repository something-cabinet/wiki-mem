//! CLI binary smoke tests: exit codes, stdout contracts, and the stdin pipe.
//!
//! Behavioral contracts (page/search/task/time/lint/validate semantics) live
//! in the in-process suites (mcp_test, e2e_*); this file keeps only the thin
//! seam the full binary owns — arg parsing, exit codes, JSON stdout shape, and
//! platform-config file writing.

#[path = "helpers/cli.rs"]
mod helpers;
use helpers::{run_cli, run_cli_with_stdin};

#[path = "helpers/setup.rs"]
mod setup;
use setup::setup_test_project;

#[path = "helpers/macros.rs"]
mod _macros;

#[test]
fn test_cli_help_and_version() {
    let (_dir, root) = setup_test_project();
    let res = run_cli(&root, &["--help"]);
    assert_success!(res);
    assert_contains!(res.stdout, "Usage:");
    let res = run_cli(&root, &["version"]);
    assert_success!(res);
}

#[test]
fn test_cli_page_list_json_contract() {
    let (_dir, root) = setup_test_project();
    let res = run_cli(&root, &["page", "list", "--json"]);
    assert_success!(res);
    let parsed: serde_json::Value =
        serde_json::from_str(&res.stdout).expect("page list --json should be valid JSON");
    assert!(parsed.get("pages").is_some());
    assert_eq!(
        parsed.get("total").and_then(|v| v.as_u64()),
        Some(0),
        "expected 0 pages in an empty project"
    );
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

    let res = run_cli(&root, &["page", "get", "wiki:concepts:test-concept"]);
    assert_success!(res);
    assert_contains!(res.stdout, "Test Concept");
}

#[test]
fn test_cli_validate_json_contract() {
    let (_dir, root) = setup_test_project();
    let res = run_cli(&root, &["validate", "--json"]);
    assert_success!(res);
    let parsed: serde_json::Value =
        serde_json::from_str(&res.stdout).expect("validate --json should be valid JSON");
    assert!(parsed.get("status").is_some());
    assert!(parsed.get("nodes").is_some());
}

#[test]
fn test_cli_lint_check() {
    let (_dir, root) = setup_test_project();
    let res = run_cli(&root, &["lint", "check"]);
    assert_success!(res);
    assert_contains!(res.stdout, "Nodes");
}

#[test]
fn test_cli_index_rebuild() {
    let (_dir, root) = setup_test_project();
    run_cli_with_stdin(
        &root,
        &["page", "create", "concepts/index-rebuild-test", "Index Rebuild Test"],
        "Body for rebuild.",
    );
    let res = run_cli(&root, &["index", "rebuild"]);
    assert_success!(res);
    assert_contains!(res.stdout, "Rebuild complete.");
}

/// `wm setup all` must write every platform's MCP config referencing the wm
/// server, and `wm agents --sync` must write the agent handbook shims.
#[test]
fn test_setup_all_and_agents_sync() {
    let (_dir, root) = setup_test_project();
    let res = run_cli(&root, &["setup", "all"]);
    assert_success!(res);
    for path in [
        "opencode.json",
        ".mcp.json",
        ".kiro/settings/mcp.json",
        ".codex/config.toml",
        ".cursor/mcp.json",
        ".gemini/antigravity/mcp_config.json",
    ] {
        assert!(root.join(path).exists(), "{path} should exist");
    }
    let opencode = std::fs::read_to_string(root.join("opencode.json")).unwrap();
    let parsed: serde_json::Value =
        serde_json::from_str(&opencode).expect("opencode.json should be valid JSON");
    assert!(parsed.pointer("/mcp/wm").is_some(), "opencode.json should have mcp.wm");
    let codex = std::fs::read_to_string(root.join(".codex/config.toml")).unwrap();
    assert!(codex.contains("[mcp_servers.wm]"), "codex config should have mcp_servers.wm");

    let res = run_cli(&root, &["agents", "--sync"]);
    assert_success!(res);
    for path in ["CLAUDE.md", "AGENTS.md", "GEMINI.md", ".github/copilot-instructions.md"] {
        assert!(root.join(path).exists(), "{path} should exist");
    }
}

/// `--content` was removed from the CLI surface; clap must reject it.
#[test]
fn test_regression_content_flag_rejected() {
    let (_dir, root) = setup_test_project();
    let res = run_cli(
        &root,
        &["page", "create", "concepts/no-flag", "No Flag", "--content", "x"],
    );
    assert_ne!(res.exit_code, 0, "expected --content to be rejected by clap");
    assert_contains!(res.stderr, "unexpected argument");
}

#[test]
fn test_regression_page_update_tags() {
    let (_dir, root) = setup_test_project();
    run_cli_with_stdin(&root, &["page", "create", "regression/tags", "Tags Test"], "content");
    let res = run_cli_with_stdin(
        &root,
        &["page", "update", "wiki:regression:tags"],
        r#"{"tags": ["rust", "async", "test"]}"#,
    );
    assert_success!(res);
    let res = run_cli(&root, &["page", "get", "wiki:regression:tags", "--json"]);
    assert_success!(res);
    assert_contains!(res.stdout, "rust");
    assert_contains!(res.stdout, "async");
    assert_contains!(res.stdout, "test");
}

#[test]
fn test_cli_time_tracking_persists_frontmatter() {
    let (_dir, root) = setup_test_project();
    run_cli_with_stdin(
        &root,
        &["page", "create", "tasks/time-tracked-task", "Time Tracked Task"],
        "Body.",
    );
    let res = run_cli(&root, &["time", "start", "wiki:tasks:time-tracked-task", "--json"]);
    assert_success!(res);
    let res = run_cli(&root, &["time", "stop", "wiki:tasks:time-tracked-task", "--json"]);
    assert_success!(res);

    let content =
        std::fs::read_to_string(root.join(".wm/wiki/tasks/time-tracked-task.md")).unwrap_or_default();
    assert!(content.contains("time_spent:"), "time stop must persist time_spent");
    assert!(content.contains("time_started:"), "time start must persist time_started");

    let res = run_cli(&root, &["time", "report", "--json"]);
    assert_success!(res);
    assert_contains!(res.stdout, "time-tracked-task");
}

/// `wm index code` must build the code-intel database from the project tree.
#[test]
fn test_cli_index_code_builds_db() {
    let (_dir, root) = setup_test_project();
    std::fs::create_dir_all(root.join("src")).expect("create src");
    std::fs::write(root.join("src/lib.rs"), "pub fn cli_func() -> u32 { 99 }")
        .expect("write source");
    let res = run_cli(&root, &["index", "code"]);
    assert_success!(res);
    assert!(
        root.join(".wm/state/code.db").exists(),
        "code.db should exist after wm index code"
    );
}

/// `wm health audit --fix` must stub referenced empty tasks and delete orphan
/// empty tasks while leaving non-empty tasks untouched.
#[test]
fn test_health_fix_stubs_and_deletes() {
    let (_dir, root) = setup_test_project();
    let wiki_dir = root.join(".wm").join("wiki");
    std::fs::write(
        wiki_dir.join("tasks/stub-me.md"),
        "---\ntitle: Stub Me\ntype: task\nstatus: todo\n---\n",
    )
    .unwrap();
    std::fs::write(
        wiki_dir.join("tasks/orphan-me.md"),
        "---\ntitle: Orphan Me\ntype: task\nstatus: todo\n---\n",
    )
    .unwrap();
    std::fs::write(
        wiki_dir.join("tasks/has-body.md"),
        "---\ntitle: Has Body\ntype: task\nstatus: in-progress\n---\n\n## Overview\n\nReal content.\n",
    )
    .unwrap();
    std::fs::write(
        wiki_dir.join("concepts/refs.md"),
        "---\ntitle: Refs\ntype: concept\nrelates_to:\n  - {type: references, target: wiki:tasks:stub-me}\n---\n\n## Overview\n\nRefs.\n",
    )
    .unwrap();

    let res = run_cli(&root, &["health", "audit", "--fix"]);
    assert_success!(res);
    assert_contains!(res.stdout, "1 pages stubbed");
    assert_contains!(res.stdout, "1 pages deleted");

    let stub = std::fs::read_to_string(wiki_dir.join("tasks/stub-me.md")).unwrap();
    assert_contains!(stub, "## Overview");
    assert_contains!(stub, "Task stub");
    assert!(!wiki_dir.join("tasks/orphan-me.md").exists(), "orphan must be deleted");
    assert_contains!(std::fs::read_to_string(wiki_dir.join("tasks/has-body.md")).unwrap(), "Real content.");
}

/// AC-4.1 (query-before-grep hooks, strict): `wm setup opencode --strict` must
/// produce an opencode.json whose first raw read is permission-gated — a
/// permission rule that requires approval for the raw file-read tools
/// (`read`/`grep`/`bash`) — plus the query-before-grep instruction file
/// referenced from the config. Verified by config inspection (fixture run).
#[test]
fn test_setup_opencode_strict_gates_first_read() {
    let (_dir, root) = setup_test_project();
    let res = run_cli(&root, &["setup", "opencode", "--strict"]);
    assert_success!(res);

    let opencode = std::fs::read_to_string(root.join("opencode.json")).unwrap();
    let parsed: serde_json::Value =
        serde_json::from_str(&opencode).expect("opencode.json should be valid JSON");

    let instructions = parsed["instructions"]
        .as_array()
        .expect("opencode.json should have an instructions array");
    assert!(
        instructions
            .iter()
            .any(|v| v.as_str() == Some("./.wm/agent/query-before-grep.md")),
        "instructions must reference the query-before-grep hook file"
    );

    let permission = parsed["permission"]
        .as_object()
        .expect("strict setup must emit a permission block");
    for tool in ["read", "grep", "bash"] {
        assert_eq!(
            permission[tool],
            "ask",
            "strict mode must gate {tool} (raw file read surface) behind approval"
        );
    }

    let hook = root.join(".wm/agent/query-before-grep.md");
    assert!(hook.exists(), "query-before-grep instruction file should exist");
    let hook_content = std::fs::read_to_string(&hook).unwrap();
    assert_contains!(hook_content, "wm_graph");
    assert_contains!(hook_content, "wm_search");
    assert!(
        hook_content.contains("Strict mode"),
        "strict hook file must carry the enforcement note"
    );
}

/// AC-4.2 (query-before-grep hooks, non-strict): `wm setup opencode` must emit
/// the guidance as instructions only — the instruction file is referenced, but
/// no permission/enforcement block is written.
#[test]
fn test_setup_opencode_non_strict_instruction_only() {
    let (_dir, root) = setup_test_project();
    let res = run_cli(&root, &["setup", "opencode"]);
    assert_success!(res);

    let opencode = std::fs::read_to_string(root.join("opencode.json")).unwrap();
    let parsed: serde_json::Value =
        serde_json::from_str(&opencode).expect("opencode.json should be valid JSON");

    let instructions = parsed["instructions"]
        .as_array()
        .expect("opencode.json should have an instructions array");
    assert!(
        instructions
            .iter()
            .any(|v| v.as_str() == Some("./.wm/agent/query-before-grep.md")),
        "non-strict setup must still reference the instruction file"
    );
    assert!(
        parsed.get("permission").is_none(),
        "non-strict setup must NOT emit a permission/enforcement block"
    );

    let hook = root.join(".wm/agent/query-before-grep.md");
    assert!(hook.exists(), "query-before-grep instruction file should exist");
    let hook_content = std::fs::read_to_string(&hook).unwrap();
    assert_contains!(hook_content, "wm_graph");
    assert_contains!(hook_content, "wm_search");
    assert!(
        !hook_content.contains("Strict mode"),
        "non-strict hook file must be instructions only (no enforcement note)"
    );
}

/// Re-running `wm setup opencode` must stay idempotent: the instruction
/// reference must not be duplicated in the instructions array.
#[test]
fn test_setup_opencode_hook_is_idempotent() {
    let (_dir, root) = setup_test_project();
    for _ in 0..2 {
        let res = run_cli(&root, &["setup", "opencode", "--strict"]);
        assert_success!(res);
    }
    let opencode = std::fs::read_to_string(root.join("opencode.json")).unwrap();
    let parsed: serde_json::Value =
        serde_json::from_str(&opencode).expect("opencode.json should be valid JSON");
    let refs = parsed["instructions"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|v| v.as_str() == Some("./.wm/agent/query-before-grep.md"))
        .count();
    assert_eq!(refs, 1, "instruction reference must not be duplicated");
}

/// Claude Code strict: `.claude/settings.json` permission rules gate the first
/// raw read (Read) and shell commands (Bash), and CLAUDE.md imports the shared
/// instruction file. Non-enforceable platforms (e.g. cursor) only get the
/// instruction file.
#[test]
fn test_setup_claude_strict_writes_settings_and_import() {
    let (_dir, root) = setup_test_project();
    let res = run_cli(&root, &["setup", "claude", "--strict"]);
    assert_success!(res);

    let settings = std::fs::read_to_string(root.join(".claude/settings.json")).unwrap();
    let parsed: serde_json::Value =
        serde_json::from_str(&settings).expect(".claude/settings.json should be valid JSON");
    let ask = parsed["permissions"]["ask"]
        .as_array()
        .expect("settings should have an ask rule list");
    assert!(ask.iter().any(|v| v.as_str() == Some("Read(//**)")), "Read must be gated");
    assert!(ask.iter().any(|v| v.as_str() == Some("Bash(*)")), "Bash must be gated");

    let claude = std::fs::read_to_string(root.join("CLAUDE.md")).unwrap();
    assert_contains!(claude, "@.wm/agent/query-before-grep.md");
}

/// Instruction-only platforms: cursor emits a rule file (instruction surface)
/// but no enforcement mechanism exists.
#[test]
fn test_setup_cursor_emits_instruction_rule_only() {
    let (_dir, root) = setup_test_project();
    let res = run_cli(&root, &["setup", "cursor", "--strict"]);
    assert_success!(res);

    let rule = root.join(".cursor/rules/query-before-grep.mdc");
    assert!(rule.exists(), "cursor should get a query-before-grep rule file");
    let content = std::fs::read_to_string(&rule).unwrap();
    assert_contains!(content, "wm_graph");
    assert_contains!(content, "wm_search");
    assert!(
        !root.join(".cursor/settings.json").exists(),
        "cursor has no permission-rule mechanism; strict must be instruction-only"
    );
}
