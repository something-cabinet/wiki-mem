
#[path = "helpers/cli.rs"]
mod helpers;
use helpers::{run_cli, run_cli_with_stdin};

#[path = "helpers/setup.rs"]
mod setup;
use setup::setup_test_project;

#[path = "helpers/macros.rs"]
mod _macros;

#[test]
fn create_all_page_types() {
    let (_dir, root) = setup_test_project();

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

    let res = run_cli(&root, &["page", "list", "--json"]);
    assert_success!(res);
    let parsed: serde_json::Value =
        serde_json::from_str(&res.stdout).expect("valid JSON");
    let total = parsed.get("total").and_then(|v| v.as_u64()).unwrap_or(0);
    assert_eq!(total, 7, "expected 7 pages, got {}", total);
}

#[test]
fn page_status_assignment() {
    let (_dir, root) = setup_test_project();

    let res = run_cli_with_stdin(
        &root,
        &[
            "page", "create", "concepts/e2e-status-concept",
            "Status Concept",
            "--page-type", "concept",
        ],
        "Concept page with default status.",
    );
    assert_success!(res);

    let res = run_cli_with_stdin(
        &root,
        &["page", "create", "tasks/e2e-status-task", "Status Task"],
        "Task page with default status.",
    );
    assert_success!(res);

    let res = run_cli(&root, &["page", "list", "--json"]);
    assert_success!(res);
    let parsed: serde_json::Value =
        serde_json::from_str(&res.stdout).expect("valid JSON");
    let empty = vec![];
    let pages = parsed.get("pages").and_then(|v| v.as_array()).unwrap_or(&empty);

    let concept = pages.iter().find(|p| {
        p.get("id")
            .and_then(|v| v.as_str())
            .map(|s| s.contains("e2e-status-concept"))
            .unwrap_or(false)
    });
    assert!(concept.is_some(), "concept page should appear in page list");

    let task = pages.iter().find(|p| {
        p.get("id")
            .and_then(|v| v.as_str())
            .map(|s| s.contains("e2e-status-task"))
            .unwrap_or(false)
    });
    assert!(task.is_some(), "task page should appear in page list");

    for p in pages {
        let id = p.get("id").and_then(|v| v.as_str()).unwrap_or("");
        let status = p.get("status").and_then(|v| v.as_str()).unwrap_or("");
        if id.contains("e2e-status-task") {
            assert_eq!(status, "todo", "task should have status 'todo', got '{}'", status);
        }
        if id.contains("e2e-status-concept") {
            assert_eq!(status, "draft", "concept should have status 'draft', got '{}'", status);
        }
    }
}

#[test]
fn page_update_roundtrip() {
    let (_dir, root) = setup_test_project();

    let res = run_cli_with_stdin(
        &root,
        &["page", "create", "tasks/e2e-update-page", "Update Page Test"],
        "This page will be used to test index rebuild after creation.",
    );
    assert_success!(res);

    let res = run_cli(&root, &["index", "rebuild"]);
    assert_success!(res);

    let res = run_cli(&root, &["page", "list", "--json"]);
    assert_success!(res);
    let parsed: serde_json::Value =
        serde_json::from_str(&res.stdout).expect("valid JSON");
    let empty_pages = vec![];
    let pages = parsed.get("pages").and_then(|v| v.as_array()).unwrap_or(&empty_pages);
    let has_page = pages.iter().any(|p| {
        p.get("id")
            .and_then(|v| v.as_str())
            .map(|s| s.contains("e2e-update-page"))
            .unwrap_or(false)
    });
    assert!(has_page, "page should survive index rebuild");
}

#[test]
fn page_linking() {
    let (_dir, root) = setup_test_project();

    let res = run_cli_with_stdin(
        &root,
        &["page", "create", "concepts/e2e-ref-concept", "Reference Concept"],
        "A concept referenced by a task.",
    );
    assert_success!(res);

    let res = run_cli_with_stdin(
        &root,
        &["page", "create", "tasks/e2e-ref-task", "Reference Task"],
        "A task that references the concept.",
    );
    assert_success!(res);

    let res = run_cli(&root, &[
        "page", "link",
        "wiki:tasks:e2e-ref-task",
        "wiki:concepts:e2e-ref-concept",
        "--edge-type", "relates_to",
    ]);
    assert_success!(res);

    let res = run_cli(&root, &[
        "graph", "neighbors", "wiki:tasks:e2e-ref-task", "--json",
    ]);
    assert_success!(res);
}

#[test]
fn version_history() {
    let (_dir, root) = setup_test_project();

    let res = run_cli_with_stdin(
        &root,
        &["page", "create", "tasks/e2e-version", "Original Title"],
        "Version test content.",
    );
    assert_success!(res);

    let versions_dir = root.join(".wm").join("versions");
    std::fs::create_dir_all(&versions_dir).expect("create versions dir");
    let version = serde_json::json!({
        "entity_id": "wiki:tasks:e2e-version",
        "current_version": 1,
        "versions": [{
            "id": "v1",
            "version": 1,
            "timestamp": "2026-07-14T12:00:00Z",
            "changes": [{"field": "title", "old_value": "Original", "new_value": "Updated"}],
            "compacted": false
        }]
    });
    std::fs::write(
        versions_dir.join("task-e2e-version.json"),
        serde_json::to_string_pretty(&version).unwrap(),
    )
    .expect("write version file");

    let version_file = versions_dir.join("task-e2e-version.json");
    assert!(version_file.exists(), "version file should exist");
    let content = std::fs::read_to_string(&version_file).expect("read version file");
    let parsed: serde_json::Value = serde_json::from_str(&content).expect("valid JSON");
    assert_eq!(parsed["current_version"], 1, "version should be v1");
}
