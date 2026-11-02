#[path = "helpers/cli.rs"]
mod helpers;
use helpers::{run_cli, CliResult};

use std::path::Path;

const _: fn(&Path, &[&str], &str) -> CliResult = helpers::run_cli_with_stdin;

#[path = "helpers/setup.rs"]
mod setup;
use setup::setup_test_project;

const EXIT_OK: i32 = 0;
const EXIT_ERROR: i32 = 1;
const MISSING_ID: &str = "does-not-exist";
const MISSING_OTHER_ID: &str = "does-not-exist-either";
const NOT_FOUND_CODE: &str = "NOT_FOUND";
#[cfg(unix)]
const READ_ONLY_DIR_MODE: u32 = 0o555;

fn failing_commands() -> Vec<Vec<&'static str>> {
    vec![
        vec!["graph", "neighbors", MISSING_ID],
        vec!["graph", "subgraph", MISSING_ID],
        vec!["graph", "affected", MISSING_ID],
        vec!["graph", "path", MISSING_ID, MISSING_OTHER_ID],
        vec!["page", "get", MISSING_ID],
        vec!["page", "delete", MISSING_ID],
        vec!["source", "status", MISSING_ID],
        vec!["source", "remove", MISSING_ID],
        vec!["time", "start", MISSING_ID],
        vec!["time", "stop", MISSING_ID],
        vec!["time", "add", MISSING_ID, "1h"],
    ]
}

fn succeeding_commands() -> Vec<Vec<&'static str>> {
    vec![
        vec!["page", "list"],
        vec!["graph", "stats"],
        vec!["task", "board"],
        vec!["lint", "check"],
        vec!["validate"],
    ]
}

#[test]
fn failing_tool_calls_exit_non_zero_with_the_error_on_stderr() {
    let (_dir, root) = setup_test_project();
    let mut failures = Vec::new();

    for args in failing_commands() {
        let res = run_cli(&root, &args);
        let named_on_stderr = res.stderr.contains(NOT_FOUND_CODE);
        let silent_on_stdout = res.stdout.trim().is_empty();
        if res.exit_code != EXIT_ERROR || !named_on_stderr || !silent_on_stdout {
            failures.push(format!(
                "{}: exit {} stdout {:?} stderr {:?}",
                args.join(" "),
                res.exit_code,
                res.stdout,
                res.stderr
                    .lines()
                    .filter(|line| !line.contains(" INFO ") && !line.contains(" WARN "))
                    .collect::<Vec<_>>()
            ));
        }
    }

    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn succeeding_tool_calls_still_exit_zero() {
    let (_dir, root) = setup_test_project();
    let mut failures = Vec::new();

    for args in succeeding_commands() {
        let res = run_cli(&root, &args);
        if res.exit_code != EXIT_OK {
            failures.push(format!(
                "{}: exit {} stderr {:?}",
                args.join(" "),
                res.exit_code,
                res.stderr
            ));
        }
    }

    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn obsidian_export_without_out_exits_non_zero() {
    let (_dir, root) = setup_test_project();

    let res = run_cli(&root, &["graph", "export", "obsidian"]);

    assert_eq!(res.exit_code, EXIT_ERROR, "stderr: {}", res.stderr);
    assert!(
        res.stderr.contains("requires --out"),
        "stderr: {}",
        res.stderr
    );
    assert!(res.stdout.trim().is_empty(), "stdout: {}", res.stdout);
}

#[cfg(unix)]
#[test]
fn health_fix_that_cannot_delete_exits_non_zero() {
    use std::os::unix::fs::PermissionsExt;

    let (_dir, root) = setup_test_project();
    let tasks_dir = root.join(".wm/wiki/tasks");
    std::fs::write(
        tasks_dir.join("orphan-me.md"),
        "---\ntitle: Orphan Me\ntype: task\nstatus: todo\n---\n",
    )
    .expect("write orphan task");
    let writable = std::fs::metadata(&tasks_dir)
        .expect("tasks metadata")
        .permissions();
    let mut read_only = writable.clone();
    read_only.set_mode(READ_ONLY_DIR_MODE);
    std::fs::set_permissions(&tasks_dir, read_only).expect("make tasks read-only");

    let res = run_cli(&root, &["health", "audit", "--fix"]);

    std::fs::set_permissions(&tasks_dir, writable).expect("restore tasks permissions");
    assert_eq!(
        res.exit_code, EXIT_ERROR,
        "stdout: {}\nstderr: {}",
        res.stdout, res.stderr
    );
    assert!(
        res.stderr.contains("Failed to delete"),
        "stderr: {}",
        res.stderr
    );
    assert!(
        tasks_dir.join("orphan-me.md").exists(),
        "the page that could not be deleted must still exist"
    );
}
