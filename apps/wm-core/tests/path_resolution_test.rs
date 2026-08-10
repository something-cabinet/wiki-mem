#[path = "helpers/cli.rs"]
mod helpers;
use helpers::{run_cli, run_cli_with_stdin};

#[path = "helpers/setup.rs"]
mod setup;
use setup::setup_test_project;

#[path = "helpers/macros.rs"]
mod _macros;

#[test]
fn graph_meta_path_is_relative_to_project_root() {
    let (_dir, root) = setup_test_project();
    let wiki_dir = root.join(".wm").join("wiki");
    std::fs::create_dir_all(wiki_dir.join("tasks")).expect("create tasks dir");
    std::fs::write(
        wiki_dir.join("tasks/rel-path-task.md"),
        "# Rel Path Task\n\nBody.",
    )
    .expect("write page");

    let (graph, _id_index) = wm_core::graph::build_graph_from_wiki(&wiki_dir, &[]);
    let meta = graph
        .node_weights()
        .find(|m| m.id == "wiki:tasks:rel-path-task")
        .expect("page node should exist in graph");
    let rel = meta.path.to_string_lossy();
    assert!(
        !meta.path.is_absolute(),
        "meta.path must be relative to project root, got '{}'",
        rel
    );
    assert!(
        rel.starts_with(".wm/wiki/"),
        "meta.path must start with '.wm/wiki/', got '{}'",
        rel
    );
    assert!(
        root.join(&meta.path).exists(),
        "meta.path '{}' should resolve relative to project root",
        rel
    );
}

#[test]
fn cli_page_crud_from_wiki_dir_cwd_resolves_meta_path() {
    let (_dir, root) = setup_test_project();
    let wiki_dir = root.join(".wm").join("wiki");
    std::fs::create_dir_all(wiki_dir.join("tasks")).expect("create tasks dir");
    std::fs::create_dir_all(wiki_dir.join("concepts")).expect("create concepts dir");

    // Create pages from the project-root CWD so they land in the real wiki dir.
    let res = run_cli_with_stdin(
        &root,
        &[
            "page",
            "create",
            "concepts/wiki-cwd-concept",
            "Wiki Cwd Concept",
        ],
        "Concept body for wiki-dir CWD resolution.",
    );
    assert_success!(res);
    let res = run_cli_with_stdin(
        &root,
        &["page", "create", "tasks/wiki-cwd-task", "Wiki Cwd Task"],
        "Task body for wiki-dir CWD resolution.",
    );
    assert_success!(res);

    // Regression: run link/update/delete with CWD = .wm/wiki — meta.path must
    // be anchored to the project root, never double-prefixed.
    let res = run_cli_with_stdin(
        &wiki_dir,
        &["page", "update", "wiki:tasks:wiki-cwd-task"],
        r#"{"title": "Wiki Cwd Task Updated", "status": "in-progress"}"#,
    );
    assert_success!(res);

    let res = run_cli(&wiki_dir, &["page", "get", "wiki:tasks:wiki-cwd-task", "--json"]);
    assert_success!(res);
    let parsed: serde_json::Value = serde_json::from_str(&res.stdout).expect("valid JSON");
    let content = parsed["content"].as_str().unwrap_or("");
    assert_contains!(content, "Wiki Cwd Task Updated");
    assert_contains!(content, "status: in-progress");

    let res = run_cli(
        &wiki_dir,
        &[
            "page",
            "link",
            "wiki:tasks:wiki-cwd-task",
            "wiki:concepts:wiki-cwd-concept",
        ],
    );
    assert_success!(res);

    let res = run_cli(
        &wiki_dir,
        &[
            "page",
            "unlink",
            "wiki:tasks:wiki-cwd-task",
            "wiki:concepts:wiki-cwd-concept",
        ],
    );
    assert_success!(res);

    let res = run_cli(&wiki_dir, &["page", "delete", "wiki:tasks:wiki-cwd-task"]);
    assert_success!(res);
    let deleted = wiki_dir.join("tasks").join("wiki-cwd-task.md");
    assert!(
        !deleted.exists(),
        "delete from .wm/wiki CWD should remove the page file at {}",
        deleted.display()
    );
}

#[test]
fn cli_page_crud_from_wiki_root_resolves_meta_path() {
    let (_dir, root) = setup_test_project();

    let res = run_cli_with_stdin(
        &root,
        &[
            "page",
            "create",
            "concepts/e2e-root-concept",
            "Root Concept",
        ],
        "Concept body for root CWD resolution.",
    );
    assert_success!(res);

    let res = run_cli_with_stdin(
        &root,
        &["page", "create", "tasks/e2e-root-task", "Root Task"],
        "Task body for root CWD resolution.",
    );
    assert_success!(res);

    let res = run_cli(
        &root,
        &["page", "get", "wiki:tasks:e2e-root-task", "--json"],
    );
    assert_success!(res);
    let parsed: serde_json::Value = serde_json::from_str(&res.stdout).expect("valid JSON");
    let content = parsed["content"].as_str().unwrap_or("");
    assert_contains!(content, "Task body for root CWD resolution.");

    let res = run_cli_with_stdin(
        &root,
        &["page", "update", "wiki:tasks:e2e-root-task"],
        r#"{"title": "Root Task Updated", "status": "in-progress"}"#,
    );
    assert_success!(res);

    let res = run_cli(
        &root,
        &["page", "get", "wiki:tasks:e2e-root-task", "--json"],
    );
    assert_success!(res);
    let parsed: serde_json::Value = serde_json::from_str(&res.stdout).expect("valid JSON");
    let content = parsed["content"].as_str().unwrap_or("");
    assert_contains!(content, "status: in-progress");

    let res = run_cli(
        &root,
        &[
            "page",
            "link",
            "wiki:tasks:e2e-root-task",
            "wiki:concepts:e2e-root-concept",
        ],
    );
    assert_success!(res);

    let res = run_cli(
        &root,
        &[
            "page",
            "unlink",
            "wiki:tasks:e2e-root-task",
            "wiki:concepts:e2e-root-concept",
        ],
    );
    assert_success!(res);

    let res = run_cli(&root, &["page", "delete", "wiki:tasks:e2e-root-task"]);
    assert_success!(res);
    let deleted = root
        .join(".wm")
        .join("wiki")
        .join("tasks")
        .join("e2e-root-task.md");
    assert!(
        !deleted.exists(),
        "delete should remove the page file at {}",
        deleted.display()
    );
}
