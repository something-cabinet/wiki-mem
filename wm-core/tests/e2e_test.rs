// ─── Full Workflow E2E Test ───────────────────────────────────
// Simulates a complete agent session:
//   init → create pages → add relates_to links → search → retrieve →
//   graph exploration → create task → time tracking → update status →
//   lint → validate → rebuild index

mod helpers;

use helpers::{run_cli, setup_test_project};

#[test]
fn test_full_workflow() {
    let (_dir, root) = setup_test_project();

    // ─── Step 1: Create pages (all 7 types) ───────────────────

    // Task
    let res = run_cli(&root, &[
        "page", "create", "tasks/e2e-task",
        "E2E Task: Implement Feature",
        "--content", "Implement the main feature for E2E testing.",
    ]);
    assert_success!(res);

    // Spec
    let res = run_cli(&root, &[
        "page", "create", "specs/e2e-spec",
        "E2E Spec",
        "--content", "# E2E Spec\n\nFR-1: The system must work.",
    ]);
    assert_success!(res);

    // Concept
    let res = run_cli(&root, &[
        "page", "create", "concepts/e2e-concept",
        "E2E Concept",
        "--content", "A concept for E2E testing.",
    ]);
    assert_success!(res);

    // Pattern
    let res = run_cli(&root, &[
        "page", "create", "patterns/e2e-pattern",
        "E2E Pattern",
        "--content", "A reusable pattern for E2E testing.",
    ]);
    assert_success!(res);

    // Decision
    let res = run_cli(&root, &[
        "page", "create", "decisions/e2e-decision",
        "E2E Decision",
        "--content", "We decided to use Rust for E2E testing.",
    ]);
    assert_success!(res);

    // Howto
    let res = run_cli(&root, &[
        "page", "create", "howto/e2e-howto",
        "E2E Howto",
        "--content", "Step 1: Run tests.",
    ]);
    assert_success!(res);

    // Reference
    let res = run_cli(&root, &[
        "page", "create", "reference/e2e-reference",
        "E2E Reference",
        "--content", "API reference for E2E testing.",
    ]);
    assert_success!(res);

    // ─── Step 2: Verify all pages listed ──────────────────────

    let res = run_cli(&root, &["page", "list", "--json"]);
    assert_success!(res);
    let parsed: serde_json::Value =
        serde_json::from_str(&res.stdout).expect("valid JSON");
    let total = parsed.get("total").and_then(|v| v.as_u64()).unwrap_or(0);
    assert_eq!(total, 7, "expected 7 pages, got {}", total);

    // ─── Step 3: Link pages ──────────────────────────────────

    // Link task to spec
    let res = run_cli(&root, &[
        "page", "link",
        "wiki:tasks:e2e-task",
        "wiki:specs:e2e-spec",
        "--edge-type", "implements",
    ]);
    assert_success!(res);

    // ─── Step 4: Search across pages ──────────────────────────

    let res = run_cli(&root, &[
        "search", "query", "E2E", "--json",
    ]);
    assert_success!(res);

    // ─── Step 5: Retrieve context ─────────────────────────────

    let res = run_cli(&root, &[
        "search", "retrieve", "E2E test",
        "--token-budget", "4096", "--json",
    ]);
    assert_success!(res);

    // ─── Step 6: Graph exploration ────────────────────────────

    let res = run_cli(&root, &[
        "graph", "neighbors", "wiki:tasks:e2e-task", "--json",
    ]);
    assert_success!(res);

    let res = run_cli(&root, &["graph", "stats", "--json"]);
    assert_success!(res);
    let parsed: serde_json::Value =
        serde_json::from_str(&res.stdout).expect("valid JSON");
    let nodes = parsed.get("nodes").and_then(|v| v.as_u64()).unwrap_or(0);
    assert_eq!(nodes, 7, "expected 7 nodes, got {}", nodes);

    // ─── Step 7: Time tracking ────────────────────────────────

    // Start timer
    let res = run_cli(&root, &[
        "time", "start", "wiki:tasks:e2e-task", "--json",
    ]);
    assert_success!(res);

    // Stop timer
    let res = run_cli(&root, &[
        "time", "stop", "wiki:tasks:e2e-task", "--json",
    ]);
    assert_success!(res);

    // Time report
    let res = run_cli(&root, &["time", "report", "--json"]);
    assert_success!(res);

    // ─── Step 8: Lint and Validate ────────────────────────────

    let res = run_cli(&root, &["lint", "check", "--json"]);
    assert_success!(res);

    let res = run_cli(&root, &["validate", "--json"]);
    assert_success!(res);

    // ─── Step 9: Rebuild index ────────────────────────────────

    let res = run_cli(&root, &["index", "rebuild"]);
    assert_success!(res);

    // ─── Step 10: Verify persistence after restart ────────────

    let res = run_cli(&root, &["page", "list", "--json"]);
    assert_success!(res);
    let parsed: serde_json::Value =
        serde_json::from_str(&res.stdout).expect("valid JSON");
    let total = parsed.get("total").and_then(|v| v.as_u64()).unwrap_or(0);
    assert_eq!(total, 7, "expected 7 pages after rebuild, got {}", total);
}

#[test]
fn test_workflow_full_session() {
    let (_dir, root) = setup_test_project();

    // ─── Step 1: Init project via wm init --no-wizard ────────
    let res = run_cli(&root, &["init", "--no-wizard"]);
    assert_success!(res);

    // ─── Step 2: Create a page ────────────────────────────────
    let res = run_cli(&root, &[
        "page", "create", "tasks/test-session-task",
        "Session Task",
        "--content", "A test task for full session workflow.",
    ]);
    assert_success!(res);

    // ─── Step 3: Create a second page and add relates_to link ─
    let res = run_cli(&root, &[
        "page", "create", "specs/test-session-spec",
        "Session Spec",
        "--content", "A spec related to the session task.",
    ]);
    assert_success!(res);

    let res = run_cli(&root, &[
        "page", "link",
        "wiki:tasks:test-session-task",
        "wiki:specs:test-session-spec",
        "--edge-type", "relates_to",
    ]);
    assert_success!(res);

    // ─── Step 4: Search via wm search query ───────────────────
    let res = run_cli(&root, &[
        "search", "query", "session",
    ]);
    assert_success!(res);

    // ─── Step 5: Graph neighbors ─────────────────────────────
    let res = run_cli(&root, &[
        "graph", "neighbors", "wiki:tasks:test-session-task",
    ]);
    assert_success!(res);

    // ─── Step 6: Rebuild index ───────────────────────────────
    let res = run_cli(&root, &["index", "rebuild"]);
    assert_success!(res);

    // ─── Step 7: Validate wiki ───────────────────────────────
    let res = run_cli(&root, &["validate"]);
    assert_success!(res);

    // ─── Step 8: Lint check ──────────────────────────────────
    let res = run_cli(&root, &["lint", "check"]);
    assert_success!(res);
}

// ─── State Machine E2E Test ─────────────────────────────────────
// Creates a task, writes status transitions directly to frontmatter,
// then verifies the board reflects the correct state after each rebuild.
#[test]
fn test_state_machine_transitions() {
    let (_dir, root) = setup_test_project();

    // Create a task page with status: todo
    let task_dir = root.join(".wm").join("wiki").join("tasks");
    std::fs::create_dir_all(&task_dir).expect("create tasks dir");
    let task_path = task_dir.join("state-machine-test.md");
    std::fs::write(&task_path, "---\ntitle: State Machine Test\ntype: task\nstatus: todo\n---\n\nTest task.\n").expect("write");

    // Rebuild index
    let res = run_cli(&root, &["index", "rebuild"]);
    assert_success!(res);

    // Step 1: Task should be in "todo" column
    let res = run_cli(&root, &["task", "board", "--json"]);
    assert_success!(res);
    let parsed: serde_json::Value =
        serde_json::from_str(&res.stdout).expect("valid JSON");
    let todo = parsed.get("counts").and_then(|c| c.get("todo")).and_then(|v| v.as_u64()).unwrap_or(0);
    assert!(todo >= 1, "expected task in todo, got {}", todo);

    // Step 2: Update to in-progress and verify
    let content = std::fs::read_to_string(&task_path).expect("read");
    let updated = content.replace("status: todo", "status: in-progress");
    std::fs::write(&task_path, updated).expect("write");

    let res = run_cli(&root, &["index", "rebuild"]);
    assert_success!(res);
    let res = run_cli(&root, &["task", "board", "--json"]);
    assert_success!(res);
    let parsed: serde_json::Value =
        serde_json::from_str(&res.stdout).expect("valid JSON");
    let ip = parsed.get("counts").and_then(|c| c.get("in-progress")).and_then(|v| v.as_u64()).unwrap_or(0);
    assert!(ip >= 1, "expected task in in-progress, got {}", ip);

    // Step 3: Update to done and verify
    let content = std::fs::read_to_string(&task_path).expect("read");
    let updated = content.replace("status: in-progress", "status: done");
    std::fs::write(&task_path, updated).expect("write");

    let res = run_cli(&root, &["index", "rebuild"]);
    assert_success!(res);
    let res = run_cli(&root, &["task", "board", "--json"]);
    assert_success!(res);
    let parsed: serde_json::Value =
        serde_json::from_str(&res.stdout).expect("valid JSON");
    let dn = parsed.get("counts").and_then(|c| c.get("done")).and_then(|v| v.as_u64()).unwrap_or(0);
    assert!(dn >= 1, "expected task in done, got {}", dn);

    // Step 4: Transition done → in-progress (reopen for rework)
    let content = std::fs::read_to_string(&task_path).expect("read");
    let updated = content.replace("status: done", "status: in-progress");
    std::fs::write(&task_path, updated).expect("write");

    let res = run_cli(&root, &["index", "rebuild"]);
    assert_success!(res);
    let res = run_cli(&root, &["task", "board", "--json"]);
    assert_success!(res);
    let parsed: serde_json::Value =
        serde_json::from_str(&res.stdout).expect("valid JSON");
    let ip = parsed.get("counts").and_then(|c| c.get("in-progress")).and_then(|v| v.as_u64()).unwrap_or(0);
    assert!(ip >= 1, "expected task back in in-progress after reopen, got {}", ip);
}
