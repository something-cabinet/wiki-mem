//! Page lifecycle contracts through the real `wm-cli` binary: per-type status
//! assignment, rebuild persistence, and graph linking.

#[path = "helpers/cli.rs"]
mod helpers;
use helpers::{run_cli, run_cli_with_stdin};

#[path = "helpers/setup.rs"]
mod setup;
use setup::setup_test_project;

#[path = "helpers/macros.rs"]
mod _macros;

#[test]
fn page_status_assignment() {
    let (_dir, root) = setup_test_project();
    run_cli_with_stdin(
        &root,
        &["page", "create", "concepts/e2e-status-concept", "Status Concept"],
        "Concept page with default status.",
    );
    run_cli_with_stdin(
        &root,
        &["page", "create", "tasks/e2e-status-task", "Status Task"],
        "Task page with default status.",
    );

    let res = run_cli(&root, &["page", "list", "--json"]);
    assert_success!(res);
    let parsed: serde_json::Value = serde_json::from_str(&res.stdout).expect("valid JSON");
    let empty = vec![];
    let pages = parsed.get("pages").and_then(|v| v.as_array()).unwrap_or(&empty);
    assert_eq!(pages.len(), 2, "expected both created pages");

    for p in pages {
        let id = p.get("id").and_then(|v| v.as_str()).unwrap_or("");
        let status = p.get("status").and_then(|v| v.as_str()).unwrap_or("");
        if id.contains("e2e-status-task") {
            assert_eq!(status, "todo", "task should default to todo, got '{status}'");
        }
        if id.contains("e2e-status-concept") {
            assert_eq!(status, "draft", "concept should default to draft, got '{status}'");
        }
    }
}

#[test]
fn page_update_roundtrip() {
    let (_dir, root) = setup_test_project();
    run_cli_with_stdin(
        &root,
        &["page", "create", "tasks/e2e-update-page", "Update Page Test"],
        "Body for rebuild persistence.",
    );
    let res = run_cli(&root, &["index", "rebuild"]);
    assert_success!(res);

    let res = run_cli(&root, &["page", "list", "--json"]);
    assert_success!(res);
    let parsed: serde_json::Value = serde_json::from_str(&res.stdout).expect("valid JSON");
    let pages = parsed.get("pages").and_then(|v| v.as_array()).expect("pages");
    assert!(
        pages.iter().any(|p| p.get("id").and_then(|v| v.as_str()).is_some_and(|id| id.contains("e2e-update-page"))),
        "page must survive index rebuild"
    );
}

#[test]
fn page_linking() {
    let (_dir, root) = setup_test_project();
    run_cli_with_stdin(
        &root,
        &["page", "create", "concepts/e2e-ref-concept", "Reference Concept"],
        "A concept referenced by a task.",
    );
    run_cli_with_stdin(
        &root,
        &["page", "create", "tasks/e2e-ref-task", "Reference Task"],
        "A task that references the concept.",
    );

    let res = run_cli(
        &root,
        &[
            "page", "link", "wiki:tasks:e2e-ref-task", "wiki:concepts:e2e-ref-concept",
            "--edge-type", "relates_to",
        ],
    );
    assert_success!(res);

    let res = run_cli(&root, &["graph", "neighbors", "wiki:tasks:e2e-ref-task", "--json"]);
    assert_success!(res);
    let parsed: serde_json::Value = serde_json::from_str(&res.stdout).expect("valid JSON");
    assert!(
        parsed.get("neighbors").is_some() || parsed.get("edges").is_some(),
        "neighbors must return a neighbor list, got {parsed}"
    );
}
