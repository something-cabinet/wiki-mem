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
