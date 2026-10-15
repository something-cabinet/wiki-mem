#[path = "helpers/cli.rs"]
mod helpers;
use helpers::{run_cli, CliResult};

#[path = "helpers/setup.rs"]
mod setup;
use setup::setup_test_project;

use std::path::{Path, PathBuf};

const _: fn(&Path, &[&str], &str) -> CliResult = helpers::run_cli_with_stdin;

const GOLDEN_SUBDIR: &str = "tests/fixtures/cli_golden";
const FIXTURE_TASK_REL: &str = ".wm/wiki/tasks/task-one.md";
const FIXTURE_TASK_BODY: &str = "---\ntitle: Task One\ntype: task\nstatus: todo\npriority: high\n---\n\n## Overview\n\nOne task for characterization.\n";
const EXIT_OK: i32 = 0;
const EXIT_ERROR: i32 = 1;
const EXIT_USAGE: i32 = 2;
const CONFIG_KEY: &str = "search.default_mode";
const MISSING_CONFIG_KEY: &str = "does.not.exist";
const UNKNOWN_FLAG: &str = "--bogus";

const SUBCOMMANDS: &[&str] = &[
    "init",
    "mcp",
    "setup",
    "agents",
    "search",
    "index",
    "page",
    "graph",
    "health",
    "source",
    "task",
    "log",
    "lint",
    "validate",
    "time",
    "model",
    "decide",
    "status",
    "config",
    "version",
    "migrate-memory",
];

#[derive(Clone, Copy, PartialEq, Eq)]
enum Stream {
    Stdout,
    Stderr,
}

struct GoldenCase {
    name: String,
    args: Vec<String>,
    stream: Stream,
    exit_code: i32,
}

fn string_case(name: &str, args: &[&str], stream: Stream, exit_code: i32) -> GoldenCase {
    GoldenCase {
        name: name.to_owned(),
        args: args.iter().map(|a| (*a).to_owned()).collect(),
        stream,
        exit_code,
    }
}

fn help_cases() -> Vec<GoldenCase> {
    let mut cases = vec![string_case("help", &["--help"], Stream::Stdout, EXIT_OK)];
    for subcommand in SUBCOMMANDS {
        cases.push(string_case(
            &format!("help_{subcommand}"),
            &[subcommand, "--help"],
            Stream::Stdout,
            EXIT_OK,
        ));
    }
    cases
}

fn read_only_cases() -> Vec<GoldenCase> {
    vec![
        string_case("version", &["version"], Stream::Stdout, EXIT_OK),
        string_case("agents", &["agents"], Stream::Stdout, EXIT_OK),
        string_case("config_list", &["config", "list"], Stream::Stdout, EXIT_OK),
        string_case(
            "config_get",
            &["config", "get", CONFIG_KEY],
            Stream::Stdout,
            EXIT_OK,
        ),
        string_case("page_list", &["page", "list"], Stream::Stdout, EXIT_OK),
        string_case("graph_stats", &["graph", "stats"], Stream::Stdout, EXIT_OK),
        string_case("task_board", &["task", "board"], Stream::Stdout, EXIT_OK),
        string_case("validate", &["validate"], Stream::Stdout, EXIT_OK),
        string_case("lint_check", &["lint", "check"], Stream::Stdout, EXIT_OK),
    ]
}

fn error_cases() -> Vec<GoldenCase> {
    vec![
        string_case(
            "config_get_missing",
            &["config", "get", MISSING_CONFIG_KEY],
            Stream::Stderr,
            EXIT_ERROR,
        ),
        string_case(
            "unknown_argument",
            &["validate", UNKNOWN_FLAG],
            Stream::Stderr,
            EXIT_USAGE,
        ),
    ]
}

fn all_cases() -> Vec<GoldenCase> {
    let mut cases = help_cases();
    cases.extend(read_only_cases());
    cases.extend(error_cases());
    cases
}

fn golden_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(GOLDEN_SUBDIR)
}

fn golden_path(name: &str) -> PathBuf {
    golden_dir().join(format!("{name}.txt"))
}

fn prepare_project() -> (tempfile::TempDir, PathBuf) {
    let (dir, root) = setup_test_project();
    std::fs::write(root.join(FIXTURE_TASK_REL), FIXTURE_TASK_BODY).expect("write fixture task");
    (dir, root)
}

fn run_args(root: &Path, case: &GoldenCase) -> CliResult {
    let refs: Vec<&str> = case.args.iter().map(String::as_str).collect();
    run_cli(root, &refs)
}

fn capture(stream: Stream, result: &CliResult) -> &str {
    match stream {
        Stream::Stdout => &result.stdout,
        Stream::Stderr => &result.stderr,
    }
}

fn stream_name(stream: Stream) -> &'static str {
    match stream {
        Stream::Stdout => "stdout",
        Stream::Stderr => "stderr",
    }
}

fn check_case(root: &Path, case: &GoldenCase, failures: &mut Vec<String>) {
    let result = run_args(root, case);
    let expected = std::fs::read_to_string(golden_path(&case.name))
        .unwrap_or_else(|error| panic!("missing golden {name}: {error}", name = case.name));
    if result.exit_code != case.exit_code {
        failures.push(format!(
            "{name}: exit_code expected {expected} got {actual}\n  stderr: {stderr}",
            name = case.name,
            expected = case.exit_code,
            actual = result.exit_code,
            stderr = result.stderr,
        ));
    }
    let actual = capture(case.stream, &result);
    if actual != expected.as_str() {
        failures.push(format!(
            "{name}: {stream} mismatch\n--- expected ---\n{expected}--- actual ---\n{actual}",
            name = case.name,
            stream = stream_name(case.stream),
            expected = expected,
            actual = actual,
        ));
    }
}

#[test]
fn goldens_match_current_behaviour() {
    let (_dir, root) = prepare_project();
    let mut failures: Vec<String> = Vec::new();
    for case in all_cases() {
        check_case(&root, &case, &mut failures);
    }
    assert!(
        failures.is_empty(),
        "{count} golden mismatches:\n{details}",
        count = failures.len(),
        details = failures.join("\n"),
    );
}

#[test]
#[ignore = "refresh goldens by running the committed regenerate script"]
fn regenerate_goldens() {
    let (_dir, root) = prepare_project();
    std::fs::create_dir_all(golden_dir()).expect("create golden dir");
    for case in all_cases() {
        let result = run_args(&root, &case);
        let value = capture(case.stream, &result).to_owned();
        std::fs::write(golden_path(&case.name), value).expect("write golden");
    }
}
