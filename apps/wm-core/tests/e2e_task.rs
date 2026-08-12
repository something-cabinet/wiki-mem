//! Task-board and time-tracking contracts through the real `wm-cli` binary.

#[path = "helpers/cli.rs"]
mod helpers;
use helpers::{run_cli, run_cli_with_stdin};

#[path = "helpers/setup.rs"]
mod setup;
use setup::setup_test_project;

#[path = "helpers/macros.rs"]
mod _macros;

#[test]
fn task_board_reflects_status() {
    let (_dir, root) = setup_test_project();
    run_cli_with_stdin(
        &root,
        &["page", "create", "tasks/e2e-board-task", "Board Task"],
        "A task to verify the board structure.",
    );
    let res = run_cli(&root, &["index", "rebuild"]);
    assert_success!(res);

    let res = run_cli(&root, &["task", "board", "--json"]);
    assert_success!(res);
    let parsed: serde_json::Value = serde_json::from_str(&res.stdout).expect("valid JSON");
    let todo = parsed
        .get("counts")
        .and_then(|c| c.get("todo"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    assert!(todo >= 1, "expected at least 1 task in todo, got {}", todo);
}

#[test]
fn time_tracking_roundtrip() {
    let (_dir, root) = setup_test_project();
    run_cli_with_stdin(
        &root,
        &["page", "create", "tasks/e2e-time-task", "Time Task"],
        "Tracking time for E2E testing.",
    );
    let res = run_cli(&root, &["time", "start", "wiki:tasks:e2e-time-task", "--json"]);
    assert_success!(res);
    let res = run_cli(&root, &["time", "stop", "wiki:tasks:e2e-time-task", "--json"]);
    assert_success!(res);
    let res = run_cli(&root, &["time", "report", "--json"]);
    assert_success!(res);
    assert_contains!(res.stdout, "e2e-time-task");
}
