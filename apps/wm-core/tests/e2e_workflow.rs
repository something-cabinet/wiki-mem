//! Full-session workflow through the real `wm-cli` binary: init, create pages
//! of every type, link, search, retrieve, graph, time, lint, validate, rebuild.

#[path = "helpers/cli.rs"]
mod helpers;
use helpers::{run_cli, run_cli_with_stdin};

#[path = "helpers/setup.rs"]
mod setup;
use setup::setup_test_project;

#[path = "helpers/macros.rs"]
mod _macros;

const PAGE_CREATES: &[(&str, &str, &str)] = &[
    ("tasks/e2e-task", "E2E Task: Implement Feature", "Implement the main feature for E2E testing."),
    ("specs/e2e-spec", "E2E Spec", "# E2E Spec\n\nFR-1: The system must work."),
    ("concepts/e2e-concept", "E2E Concept", "A concept for E2E testing."),
    ("patterns/e2e-pattern", "E2E Pattern", "A reusable pattern for E2E testing."),
    ("decisions/e2e-decision", "E2E Decision", "We decided to use Rust for E2E testing."),
    ("howto/e2e-howto", "E2E Howto", "Step 1: Run tests."),
    ("reference/e2e-reference", "E2E Reference", "API reference for E2E testing."),
];

#[test]
fn full_session_workflow() {
    let (_dir, root) = setup_test_project();
    let res = run_cli(&root, &["init", "--no-wizard"]);
    assert_success!(res);

    for (path, title, body) in PAGE_CREATES {
        let res = run_cli_with_stdin(&root, &["page", "create", path, title], body);
        assert_success!(res);
    }

    let res = run_cli(&root, &["page", "list", "--json"]);
    assert_success!(res);
    let parsed: serde_json::Value = serde_json::from_str(&res.stdout).expect("valid JSON");
    assert_eq!(parsed.get("total").and_then(|v| v.as_u64()), Some(7), "expected 7 pages");

    let res = run_cli(
        &root,
        &[
            "page", "link", "wiki:tasks:e2e-task", "wiki:specs:e2e-spec",
            "--edge-type", "implements",
        ],
    );
    assert_success!(res);

    let res = run_cli(&root, &["search", "query", "E2E", "--json"]);
    assert_success!(res);

    let res = run_cli(&root, &["search", "retrieve", "E2E test", "--token-budget", "4096", "--json"]);
    assert_success!(res);

    let res = run_cli(&root, &["graph", "neighbors", "wiki:tasks:e2e-task", "--json"]);
    assert_success!(res);

    let res = run_cli(&root, &["graph", "stats", "--json"]);
    assert_success!(res);
    let parsed: serde_json::Value = serde_json::from_str(&res.stdout).expect("valid JSON");
    assert_eq!(parsed.get("nodes").and_then(|v| v.as_u64()), Some(7), "expected 7 nodes");

    let res = run_cli(&root, &["time", "start", "wiki:tasks:e2e-task", "--json"]);
    assert_success!(res);
    let res = run_cli(&root, &["time", "stop", "wiki:tasks:e2e-task", "--json"]);
    assert_success!(res);
    let res = run_cli(&root, &["time", "report", "--json"]);
    assert_success!(res);

    let res = run_cli(&root, &["lint", "check", "--json"]);
    assert_success!(res);
    let res = run_cli(&root, &["validate", "--json"]);
    assert_success!(res);

    let res = run_cli(&root, &["index", "rebuild"]);
    assert_success!(res);

    let res = run_cli(&root, &["page", "list", "--json"]);
    assert_success!(res);
    let parsed: serde_json::Value = serde_json::from_str(&res.stdout).expect("valid JSON");
    assert_eq!(parsed.get("total").and_then(|v| v.as_u64()), Some(7), "expected 7 pages after rebuild");
}
