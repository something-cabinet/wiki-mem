//! Graph contracts through the real `wm-cli` binary: link/neighbor/stats and
//! the task state machine (board counts across status transitions).

#[path = "helpers/cli.rs"]
mod helpers;
use helpers::{run_cli, run_cli_with_stdin};

#[path = "helpers/setup.rs"]
mod setup;
use setup::setup_test_project;

#[path = "helpers/macros.rs"]
mod _macros;

#[test]
fn graph_stats_and_neighbors() {
    let (_dir, root) = setup_test_project();
    run_cli_with_stdin(
        &root,
        &["page", "create", "concepts/e2e-graph-concept", "Graph Concept"],
        "A concept for graph testing.",
    );
    run_cli_with_stdin(
        &root,
        &["page", "create", "tasks/e2e-graph-task", "Graph Task"],
        "A task linked to the concept.",
    );
    let res = run_cli(
        &root,
        &[
            "page", "link", "wiki:tasks:e2e-graph-task", "wiki:concepts:e2e-graph-concept",
            "--edge-type", "relates_to",
        ],
    );
    assert_success!(res);

    let res = run_cli(&root, &["graph", "neighbors", "wiki:tasks:e2e-graph-task", "--json"]);
    assert_success!(res);

    let res = run_cli(&root, &["graph", "stats", "--json"]);
    assert_success!(res);
    let parsed: serde_json::Value = serde_json::from_str(&res.stdout).expect("valid JSON");
    let nodes = parsed.get("nodes").and_then(|v| v.as_u64()).unwrap_or(0);
    assert!(nodes >= 1, "expected at least 1 graph node, got {}", nodes);
    assert!(parsed.get("types").is_some(), "graph stats should include 'types'");
}

/// Board counts must track a task's status transitions across rebuilds.
#[test]
fn state_machine_transitions() {
    let (_dir, root) = setup_test_project();
    let task_path = root.join(".wm/wiki/tasks/state-machine-test.md");
    std::fs::write(
        &task_path,
        "---\ntitle: State Machine Test\ntype: task\nstatus: todo\n---\n\nTest task.\n",
    )
    .expect("write task");

    for (from, to, column) in [
        ("todo", "in-progress", "in-progress"),
        ("in-progress", "done", "done"),
        ("done", "in-progress", "in-progress"),
    ] {
        let content = std::fs::read_to_string(&task_path).expect("read task");
        std::fs::write(&task_path, content.replace(&format!("status: {from}"), &format!("status: {to}")))
            .expect("write task");

        let res = run_cli(&root, &["index", "rebuild"]);
        assert_success!(res);
        let res = run_cli(&root, &["task", "board", "--json"]);
        assert_success!(res);
        let parsed: serde_json::Value = serde_json::from_str(&res.stdout).expect("valid JSON");
        let count = parsed
            .get("counts")
            .and_then(|c| c.get(column))
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        assert!(count >= 1, "expected task in {column}, got {}", count);
    }
}
