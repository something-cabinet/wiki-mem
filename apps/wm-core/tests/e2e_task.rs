// ─── E2E: Task & Time Operations ───────────────────────────────
// Tests task board structure and time tracking roundtrip.

mod helpers;

use helpers::{run_cli, run_cli_with_stdin, setup_test_project};

#[test]
fn task_board_reflects_status() {
    let (_dir, root) = setup_test_project();

    // Create a task page via CLI
    let res = run_cli_with_stdin(
        &root,
        &["page", "create", "tasks/e2e-board-task", "Board Task"],
        "A task to verify the board structure.",
    );
    assert_success!(res);

    // Rebuild index so task board picks it up
    let res = run_cli(&root, &["index", "rebuild"]);
    assert_success!(res);

    // Verify task board JSON has correct structure
    let res = run_cli(&root, &["task", "board", "--json"]);
    assert_success!(res);
    let parsed: serde_json::Value =
        serde_json::from_str(&res.stdout).expect("valid JSON");
    // Board should have a "counts" object with at least "todo" key
    let counts = parsed.get("counts").and_then(|v| v.as_object());
    assert!(
        counts.is_some(),
        "task board should have a 'counts' object"
    );
    let todo = parsed
        .get("counts")
        .and_then(|c| c.get("todo"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    assert!(todo >= 1, "expected at least 1 task in todo, got {}", todo);

    // Board should have a "tasks" or "columns" array
    let has_tasks = parsed.get("tasks").or_else(|| parsed.get("columns")).is_some();
    assert!(has_tasks, "task board should have 'tasks' or 'columns'");
}

#[test]
fn time_tracking_roundtrip() {
    let (_dir, root) = setup_test_project();

    // Create a task page via stdin
    let res = run_cli_with_stdin(
        &root,
        &["page", "create", "tasks/e2e-time-task", "Time Task"],
        "Tracking time for E2E testing.",
    );
    assert_success!(res);

    // Start timer
    let res = run_cli(&root, &[
        "time", "start", "wiki:tasks:e2e-time-task", "--json",
    ]);
    assert_success!(res);

    // Stop timer
    let res = run_cli(&root, &[
        "time", "stop", "wiki:tasks:e2e-time-task", "--json",
    ]);
    assert_success!(res);

    // Time report
    let res = run_cli(&root, &["time", "report", "--json"]);
    assert_success!(res);
}
