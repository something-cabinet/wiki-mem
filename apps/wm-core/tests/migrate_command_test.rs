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

const PROSE_RULE_REL: &str = ".wm/wiki/rules/prose-rule.md";
const PROSE_RULE_BODY: &str =
    "---\ntitle: Prose\ntype: rule\nstatus: active\ncategory: coding\n---\n\nBody.\n";
const EMPTY_CONCEPT_REL: &str = ".wm/wiki/concepts/empty.md";
const EMPTY_CONCEPT_BODY: &str = "---\ntitle: Empty\ntype: concept\n---\n\n";
const LEGACY_ENTRY_REL: &str = ".wm/memory/legacy-entry.json";
const LEGACY_ENTRY_JSON: &str = r#"{"id":"legacy-entry","title":"Legacy Entry","content":"Durable legacy knowledge.","tags":["legacy"],"created_at":"2026-01-01T00:00:00Z","updated_at":"2026-01-01T00:00:00Z"}"#;
const MIGRATED_PAGE_REL: &str = ".wm/wiki/memory/legacy-entry.md";
const MEMORY_HISTORY_REL: &str = ".wm/versions/doc-wiki-memory-legacy-entry.json";
const RULE_HISTORY_REL: &str = ".wm/versions/doc-wiki-rules-prose-rule.json";
const VERSIONS_DIR_REL: &str = ".wm/versions";
const NO_MEMORY_REPORT: &str = "Legacy memory: 0 entr(ies) imported";
const ONE_MEMORY_REPORT: &str = "Legacy memory: 1 entr(ies) imported";
const DRY_RUN_MEMORY_REPORT: &str = "Legacy memory: 1 entr(ies) would be imported";
const CONVERTED_ONE: &str = "Converted: 1 page(s)";
const CONVERTED_NONE: &str = "Converted: 0 page(s)";
const WOULD_CONVERT_ONE: &str = "Would convert: 1 page(s)";
const RECORD_MARKER: &str = "schema_version: 1";
const WARN_MARKER: &str = "WARN";
const EXIT_OK: i32 = 0;

fn write_prose_rule(root: &Path) {
    let path = root.join(PROSE_RULE_REL);
    std::fs::create_dir_all(path.parent().expect("rule parent")).expect("create rules dir");
    std::fs::write(path, PROSE_RULE_BODY).expect("write prose rule");
}

fn history_version(root: &Path, relative: &str) -> u64 {
    let text = std::fs::read_to_string(root.join(relative)).expect("history file");
    let parsed: serde_json::Value = serde_json::from_str(&text).expect("history json");
    parsed["current_version"].as_u64().expect("current_version")
}

#[test]
fn migrate_converts_a_prose_page_and_prints_no_engine_warnings() {
    let (_dir, root) = setup_test_project();
    write_prose_rule(&root);

    let res = run_cli(&root, &["migrate"]);

    assert_success!(res);
    assert!(res.stdout.contains(NO_MEMORY_REPORT), "stdout: {}", res.stdout);
    assert!(res.stdout.contains(CONVERTED_ONE), "stdout: {}", res.stdout);
    assert!(
        !res.stderr.contains(WARN_MARKER) && !res.stdout.contains(WARN_MARKER),
        "no engine warnings expected\nstdout: {}\nstderr: {}",
        res.stdout,
        res.stderr
    );
    let page = std::fs::read_to_string(root.join(PROSE_RULE_REL)).expect("rule");
    assert!(page.contains(RECORD_MARKER), "page: {page}");
}

#[test]
fn migrate_imports_legacy_memory_then_converts_the_new_page() {
    let (_dir, root) = setup_test_project();
    std::fs::write(root.join(LEGACY_ENTRY_REL), LEGACY_ENTRY_JSON).expect("write legacy entry");

    let res = run_cli(&root, &["migrate"]);

    assert_success!(res);
    assert!(res.stdout.contains(ONE_MEMORY_REPORT), "stdout: {}", res.stdout);
    assert!(
        !root.join(LEGACY_ENTRY_REL).exists(),
        "legacy json must be removed after migration"
    );
    let page = std::fs::read_to_string(root.join(MIGRATED_PAGE_REL)).expect("migrated page");
    assert!(page.contains(RECORD_MARKER), "page: {page}");
    assert_eq!(history_version(&root, MEMORY_HISTORY_REL), 2);
}

#[test]
fn migrate_records_a_version_for_every_page_it_rewrites() {
    let (_dir, root) = setup_test_project();
    write_prose_rule(&root);

    let res = run_cli(&root, &["migrate"]);

    assert_success!(res);
    assert_eq!(history_version(&root, RULE_HISTORY_REL), 1);
}

#[test]
fn migrate_dry_run_changes_nothing() {
    let (_dir, root) = setup_test_project();
    write_prose_rule(&root);
    std::fs::write(root.join(LEGACY_ENTRY_REL), LEGACY_ENTRY_JSON).expect("write legacy entry");

    let res = run_cli(&root, &["migrate", "--dry-run"]);

    assert_success!(res);
    assert!(res.stdout.contains(DRY_RUN_MEMORY_REPORT), "stdout: {}", res.stdout);
    assert!(res.stdout.contains(WOULD_CONVERT_ONE), "stdout: {}", res.stdout);
    assert_eq!(
        std::fs::read_to_string(root.join(PROSE_RULE_REL)).expect("rule"),
        PROSE_RULE_BODY
    );
    assert!(root.join(LEGACY_ENTRY_REL).exists());
    assert!(!root.join(VERSIONS_DIR_REL).exists());
}

#[test]
fn migrate_twice_changes_nothing_the_second_time() {
    let (_dir, root) = setup_test_project();
    write_prose_rule(&root);
    assert_eq!(run_cli(&root, &["migrate"]).exit_code, EXIT_OK);
    let after_first = std::fs::read_to_string(root.join(PROSE_RULE_REL)).expect("rule");

    let res = run_cli(&root, &["migrate"]);

    assert_success!(res);
    assert!(res.stdout.contains(CONVERTED_NONE), "stdout: {}", res.stdout);
    assert_eq!(
        std::fs::read_to_string(root.join(PROSE_RULE_REL)).expect("rule"),
        after_first
    );
    assert_eq!(history_version(&root, RULE_HISTORY_REL), 1);
}

#[test]
fn migrate_exits_non_zero_when_a_page_cannot_be_migrated() {
    let (_dir, root) = setup_test_project();
    std::fs::write(root.join(EMPTY_CONCEPT_REL), EMPTY_CONCEPT_BODY).expect("write empty concept");

    let res = run_cli(&root, &["migrate"]);

    assert_ne!(res.exit_code, EXIT_OK, "stdout: {}", res.stdout);
    assert!(
        res.stdout.contains("concepts/empty.md"),
        "skipped page must be listed\nstdout: {}",
        res.stdout
    );
}
