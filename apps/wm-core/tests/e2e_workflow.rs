// ─── E2E: Full Session Workflow ───────────────────────────────
// The big integration test simulating a complete agent session:
//   init → create pages → link → search → retrieve → graph →
//   time → lint → validate → rebuild → verify persistence

mod helpers;

use helpers::{run_cli, run_cli_with_stdin, setup_test_project};

#[test]
fn full_session_workflow() {
    let (_dir, root) = setup_test_project();

    // ─── Step 1: Init project ────────────────────────────────
    let res = run_cli(&root, &["init", "--no-wizard"]);
    assert_success!(res);

    // ─── Step 2: Create pages (all 7 types) ──────────────────
    run_cli_with_stdin(
        &root,
        &["page", "create", "tasks/e2e-task", "E2E Task: Implement Feature"],
        "Implement the main feature for E2E testing.",
    );
    run_cli_with_stdin(
        &root,
        &["page", "create", "specs/e2e-spec", "E2E Spec"],
        "# E2E Spec\n\nFR-1: The system must work.",
    );
    run_cli_with_stdin(
        &root,
        &["page", "create", "concepts/e2e-concept", "E2E Concept"],
        "A concept for E2E testing.",
    );
    run_cli_with_stdin(
        &root,
        &["page", "create", "patterns/e2e-pattern", "E2E Pattern"],
        "A reusable pattern for E2E testing.",
    );
    run_cli_with_stdin(
        &root,
        &["page", "create", "decisions/e2e-decision", "E2E Decision"],
        "We decided to use Rust for E2E testing.",
    );
    run_cli_with_stdin(
        &root,
        &["page", "create", "howto/e2e-howto", "E2E Howto"],
        "Step 1: Run tests.",
    );
    run_cli_with_stdin(
        &root,
        &["page", "create", "reference/e2e-reference", "E2E Reference"],
        "API reference for E2E testing.",
    );

    // ─── Step 3: Verify all pages listed ─────────────────────
    let res = run_cli(&root, &["page", "list", "--json"]);
    assert_success!(res);
    let parsed: serde_json::Value =
        serde_json::from_str(&res.stdout).expect("valid JSON");
    let total = parsed.get("total").and_then(|v| v.as_u64()).unwrap_or(0);
    assert_eq!(total, 7, "expected 7 pages, got {}", total);

    // ─── Step 4: Link pages ──────────────────────────────────
    let res = run_cli(&root, &[
        "page", "link",
        "wiki:tasks:e2e-task",
        "wiki:specs:e2e-spec",
        "--edge-type", "implements",
    ]);
    assert_success!(res);

    // ─── Step 5: Search query ────────────────────────────────
    let res = run_cli(&root, &["search", "query", "E2E", "--json"]);
    assert_success!(res);

    // ─── Step 6: Retrieve context ────────────────────────────
    let res = run_cli(&root, &[
        "search", "retrieve", "E2E test",
        "--token-budget", "4096", "--json",
    ]);
    assert_success!(res);

    // ─── Step 7: Graph exploration ───────────────────────────
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

    // ─── Step 8: Time tracking ───────────────────────────────
    let res = run_cli(&root, &["time", "start", "wiki:tasks:e2e-task", "--json"]);
    assert_success!(res);
    let res = run_cli(&root, &["time", "stop", "wiki:tasks:e2e-task", "--json"]);
    assert_success!(res);
    let res = run_cli(&root, &["time", "report", "--json"]);
    assert_success!(res);

    // ─── Step 9: Lint and Validate ───────────────────────────
    let res = run_cli(&root, &["lint", "check", "--json"]);
    assert_success!(res);
    let res = run_cli(&root, &["validate", "--json"]);
    assert_success!(res);

    // ─── Step 10: Rebuild index ──────────────────────────────
    let res = run_cli(&root, &["index", "rebuild"]);
    assert_success!(res);

    // ─── Step 11: Verify persistence after rebuild ───────────
    let res = run_cli(&root, &["page", "list", "--json"]);
    assert_success!(res);
    let parsed: serde_json::Value =
        serde_json::from_str(&res.stdout).expect("valid JSON");
    let total = parsed.get("total").and_then(|v| v.as_u64()).unwrap_or(0);
    assert_eq!(total, 7, "expected 7 pages after rebuild, got {}", total);
}
