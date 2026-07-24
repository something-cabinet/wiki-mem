
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

    let res = run_cli_with_stdin(
        &root,
        &["page", "create", "concepts/e2e-graph-concept", "Graph Concept"],
        "A concept for graph testing.",
    );
    assert_success!(res);

    let res = run_cli_with_stdin(
        &root,
        &["page", "create", "tasks/e2e-graph-task", "Graph Task"],
        "A task linked to the concept.",
    );
    assert_success!(res);

    let res = run_cli(&root, &[
        "page", "link",
        "wiki:tasks:e2e-graph-task",
        "wiki:concepts:e2e-graph-concept",
        "--edge-type", "relates_to",
    ]);
    assert_success!(res);

    let res = run_cli(&root, &[
        "graph", "neighbors", "wiki:tasks:e2e-graph-task", "--json",
    ]);
    assert_success!(res);

    let res = run_cli(&root, &["graph", "stats", "--json"]);
    assert_success!(res);
    let parsed: serde_json::Value =
        serde_json::from_str(&res.stdout).expect("valid JSON from graph stats");
    let nodes = parsed.get("nodes").and_then(|v| v.as_u64()).unwrap_or(0);
    assert!(nodes >= 1, "expected at least 1 graph node, got {}", nodes);

    let types = parsed.get("types").and_then(|v| v.as_object());
    assert!(
        types.is_some(),
        "graph stats should include 'types' enrichment from config"
    );

    let res = run_cli(&root, &["page", "list", "--json"]);
    assert_success!(res);
}

#[test]
fn state_machine_transitions() {
    let (_dir, root) = setup_test_project();

    let task_dir = root.join(".wm").join("wiki").join("tasks");
    std::fs::create_dir_all(&task_dir).expect("create tasks dir");
    let task_path = task_dir.join("state-machine-test.md");
    std::fs::write(
        &task_path,
        "---\ntitle: State Machine Test\ntype: task\nstatus: todo\n---\n\nTest task.\n",
    )
    .expect("write");

    let res = run_cli(&root, &["index", "rebuild"]);
    assert_success!(res);

    let res = run_cli(&root, &["task", "board", "--json"]);
    assert_success!(res);
    let parsed: serde_json::Value =
        serde_json::from_str(&res.stdout).expect("valid JSON");
    let todo = parsed
        .get("counts")
        .and_then(|c| c.get("todo"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    assert!(todo >= 1, "expected task in todo, got {}", todo);

    let content = std::fs::read_to_string(&task_path).expect("read");
    let updated = content.replace("status: todo", "status: in-progress");
    std::fs::write(&task_path, updated).expect("write");

    let res = run_cli(&root, &["index", "rebuild"]);
    assert_success!(res);
    let res = run_cli(&root, &["task", "board", "--json"]);
    assert_success!(res);
    let parsed: serde_json::Value =
        serde_json::from_str(&res.stdout).expect("valid JSON");
    let ip = parsed
        .get("counts")
        .and_then(|c| c.get("in-progress"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    assert!(ip >= 1, "expected task in in-progress, got {}", ip);

    let content = std::fs::read_to_string(&task_path).expect("read");
    let updated = content.replace("status: in-progress", "status: done");
    std::fs::write(&task_path, updated).expect("write");

    let res = run_cli(&root, &["index", "rebuild"]);
    assert_success!(res);
    let res = run_cli(&root, &["task", "board", "--json"]);
    assert_success!(res);
    let parsed: serde_json::Value =
        serde_json::from_str(&res.stdout).expect("valid JSON");
    let dn = parsed
        .get("counts")
        .and_then(|c| c.get("done"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    assert!(dn >= 1, "expected task in done, got {}", dn);

    let content = std::fs::read_to_string(&task_path).expect("read");
    let updated = content.replace("status: done", "status: in-progress");
    std::fs::write(&task_path, updated).expect("write");

    let res = run_cli(&root, &["index", "rebuild"]);
    assert_success!(res);
    let res = run_cli(&root, &["task", "board", "--json"]);
    assert_success!(res);
    let parsed: serde_json::Value =
        serde_json::from_str(&res.stdout).expect("valid JSON");
    let ip = parsed
        .get("counts")
        .and_then(|c| c.get("in-progress"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    assert!(ip >= 1, "expected task back in in-progress after reopen, got {}", ip);
}
