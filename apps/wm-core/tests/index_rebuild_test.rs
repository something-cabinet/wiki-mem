#[path = "helpers/cli.rs"]
mod helpers;
use helpers::{run_cli, CliResult};

use std::path::Path;

const _: fn(&Path, &[&str], &str) -> CliResult = helpers::run_cli_with_stdin;

#[path = "helpers/setup.rs"]
mod setup;
use setup::setup_test_project;

#[path = "helpers/macros.rs"]
mod _macros;

const FIXTURE_PAGE_REL: &str = ".wm/wiki/concepts/fixture-page.md";
const FIXTURE_PAGE_BODY: &str =
    "---\ntitle: Fixture\ntype: concept\nstatus: draft\n---\n\n## Overview\n\nA page.\n";
const GRAPH_REBUILT_MARKER: &str = "Graph rebuilt";
const EXIT_OK: i32 = 0;

fn seed_fixture_page(root: &Path) {
    let page = root.join(FIXTURE_PAGE_REL);
    std::fs::create_dir_all(page.parent().expect("page parent")).expect("create concepts dir");
    std::fs::write(page, FIXTURE_PAGE_BODY).expect("write fixture page");
}

#[test]
fn index_rebuild_builds_the_graph_once() {
    let (_dir, root) = setup_test_project();
    seed_fixture_page(&root);

    let res = run_cli(&root, &["index", "rebuild", "--skip-embed"]);

    assert_success!(res);
    let combined = format!("{}{}", res.stdout, res.stderr);
    assert_eq!(
        combined.matches(GRAPH_REBUILT_MARKER).count(),
        1,
        "the graph must be built once per run:\n{combined}"
    );
}

#[test]
fn index_rebuild_without_a_wiki_directory_exits_non_zero() {
    let (_dir, root) = setup_test_project();
    std::fs::remove_dir_all(root.join(".wm/wiki")).expect("remove wiki dir");

    let res = run_cli(&root, &["index", "rebuild", "--skip-embed"]);

    assert_ne!(
        res.exit_code, EXIT_OK,
        "stdout: {}\nstderr: {}",
        res.stdout, res.stderr
    );
    assert!(
        res.stderr.contains("wiki directory"),
        "stderr: {}",
        res.stderr
    );
    assert!(
        !res.stdout.contains("Rebuild complete"),
        "stdout: {}",
        res.stdout
    );
}

#[test]
fn index_embed_without_a_model_exits_non_zero() {
    let (_dir, root) = setup_test_project();
    seed_fixture_page(&root);

    let res = run_cli(&root, &["index", "embed"]);

    assert_ne!(
        res.exit_code, EXIT_OK,
        "stdout: {}\nstderr: {}",
        res.stdout, res.stderr
    );
    assert!(
        res.stderr.contains("embedding model"),
        "stderr: {}",
        res.stderr
    );
}

#[test]
fn index_rebuild_with_skip_embed_exits_zero_and_reports_completion() {
    let (_dir, root) = setup_test_project();
    seed_fixture_page(&root);

    let res = run_cli(&root, &["index", "rebuild", "--skip-embed"]);

    assert_eq!(res.exit_code, EXIT_OK, "stderr: {}", res.stderr);
    assert!(
        res.stdout.contains("Rebuild complete"),
        "stdout: {}",
        res.stdout
    );
}
