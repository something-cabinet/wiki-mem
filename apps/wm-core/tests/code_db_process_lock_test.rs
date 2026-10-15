#![cfg(feature = "code-intel")]

use std::path::Path;

#[path = "helpers/cli.rs"]
mod cli;
use cli::{run_cli, CliResult};

const _: fn(&Path, &[&str], &str) -> CliResult = cli::run_cli_with_stdin;

#[path = "helpers/setup.rs"]
mod setup;
use setup::setup_test_project;

fn seed_source(root: &Path) {
    let src = root.join("src");
    std::fs::create_dir_all(&src).expect("create src dir");
    std::fs::write(src.join("lib.rs"), "pub fn locked_symbol() -> u32 { 1 }\n")
        .expect("write source");
    std::fs::write(root.join(".gitignore"), "target/\nnode_modules/\n").expect("write gitignore");
}

#[test]
fn index_code_second_process_succeeds_after_in_process_code_graph_read() {
    let (_dir, root) = setup_test_project();
    seed_source(&root);

    let first = run_cli(&root, &["index", "code"]);
    assert_eq!(
        first.exit_code, 0,
        "first index code failed: {}{}",
        first.stdout, first.stderr
    );
    assert!(
        root.join(".wm/state/code.db").exists(),
        "code.db must exist after the first index code run"
    );

    let graph = wm_core::graph::code_edges::load_code_graph(&root).expect("load code graph");
    assert!(graph.is_some(), "an existing index must yield a code graph");

    let second = run_cli(&root, &["index", "code"]);
    assert_eq!(
        second.exit_code, 0,
        "a second index code must not be blocked by this process holding the code db open: {}",
        second.stderr
    );
}
