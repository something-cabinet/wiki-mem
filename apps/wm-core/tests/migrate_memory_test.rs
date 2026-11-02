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

const BROKEN_RULE_REL: &str = ".wm/wiki/rules/broken-rule.md";
const BROKEN_RULE_BODY: &str =
    "---\ntitle: Broken\ntype: rule\nstatus: active\ncategory: coding\n---\n\nBody.\n";
const LEGACY_ENTRY_REL: &str = ".wm/memory/legacy-entry.json";
const LEGACY_ENTRY_JSON: &str = r#"{"id":"legacy-entry","title":"Legacy Entry","content":"Durable legacy knowledge.","tags":["legacy"],"created_at":"2026-01-01T00:00:00Z","updated_at":"2026-01-01T00:00:00Z"}"#;
const MIGRATED_PAGE_REL: &str = ".wm/wiki/memory/legacy-entry.md";
const NO_ENTRIES_REPORT: &str = "Migrated 0 memory entries to wiki pages.";
const ONE_ENTRY_REPORT: &str = "Migrated 1 memory entries to wiki pages.";
const WARN_MARKER: &str = "WARN";

#[test]
fn migrate_memory_without_legacy_entries_prints_only_the_report() {
    let (_dir, root) = setup_test_project();
    let broken_rule = root.join(BROKEN_RULE_REL);
    std::fs::create_dir_all(broken_rule.parent().expect("rule parent")).expect("create rules dir");
    std::fs::write(broken_rule, BROKEN_RULE_BODY).expect("write broken rule");

    let res = run_cli(&root, &["migrate-memory"]);

    assert_success!(res);
    assert_eq!(
        res.stdout.trim(),
        NO_ENTRIES_REPORT,
        "stdout: {}",
        res.stdout
    );
    assert!(
        !res.stderr.contains(WARN_MARKER) && !res.stdout.contains(WARN_MARKER),
        "no engine warnings expected\nstdout: {}\nstderr: {}",
        res.stdout,
        res.stderr
    );
}

#[test]
fn migrate_memory_reports_the_entries_it_migrated() {
    let (_dir, root) = setup_test_project();
    std::fs::write(root.join(LEGACY_ENTRY_REL), LEGACY_ENTRY_JSON).expect("write legacy entry");

    let res = run_cli(&root, &["migrate-memory"]);

    assert_success!(res);
    assert_eq!(
        res.stdout.trim(),
        ONE_ENTRY_REPORT,
        "stdout: {}",
        res.stdout
    );
    assert!(
        root.join(MIGRATED_PAGE_REL).exists(),
        "migrated page must exist at {MIGRATED_PAGE_REL}"
    );
    assert!(
        !root.join(LEGACY_ENTRY_REL).exists(),
        "legacy json must be removed after migration"
    );
}
