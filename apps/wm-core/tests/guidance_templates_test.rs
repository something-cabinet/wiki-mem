#[path = "helpers/cli.rs"]
mod helpers;
use helpers::{run_cli, CliResult};

use std::path::{Path, PathBuf};

const _: fn(&Path, &[&str], &str) -> CliResult = helpers::run_cli_with_stdin;

#[path = "helpers/setup.rs"]
mod setup;
use setup::setup_test_project;

#[path = "helpers/macros.rs"]
mod _macros;

const WIKI_MEM_TEMPLATE_KEY: &str = "shims/WIKI-MEM.md";
const STEERING_TEMPLATE_KEY: &str = "steering/wiki-mem.md";
const PRIORITY_HEADING: &str = "## Priority: load rules and core docs first";
const SESSION_START_HEADING: &str = "## Session Start";
const STEERING_PRIORITY_HEADING: &str = "## Priority";
const STEERING_INCLUDE: &str = "#[[file:WIKI-MEM.md]]";
const REPO_WIKI_MEM_REL: &str = "WIKI-MEM.md";
const STEERING_OUTPUT_REL: &str = ".kiro/steering/wiki-mem.md";

const PRIORITY_PHRASES: [&str; 6] = [
    "before answering, planning or calling any other tool",
    "never skipped, deferred or replaced by a summary from memory",
    "If the wm MCP is unavailable",
    "except for safety and system instructions",
    "after a context reset",
    "after every milestone",
];

fn embedded_text(key: &str) -> String {
    let file = wm_core::embed_files::EmbeddedFiles::get(key)
        .unwrap_or_else(|| panic!("{key} must be embedded"));
    String::from_utf8(file.data.to_vec()).expect("embedded template is utf-8")
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn repo_text(rel: &str) -> String {
    std::fs::read_to_string(repo_root().join(rel))
        .unwrap_or_else(|error| panic!("read {rel}: {error}"))
}

#[test]
fn wiki_mem_template_leads_with_the_priority_section() {
    let template = embedded_text(WIKI_MEM_TEMPLATE_KEY);

    let priority = template.find(PRIORITY_HEADING).expect("priority heading");
    let session_start = template
        .find(SESSION_START_HEADING)
        .expect("session start heading");
    assert!(priority < session_start, "priority must come first");
    for phrase in PRIORITY_PHRASES {
        assert!(template.contains(phrase), "missing: {phrase}");
    }
}

#[test]
fn kiro_steering_template_names_the_priority_before_including_wiki_mem() {
    let template = embedded_text(STEERING_TEMPLATE_KEY);

    let priority = template
        .find(STEERING_PRIORITY_HEADING)
        .expect("steering priority heading");
    let include = template.find(STEERING_INCLUDE).expect("wiki-mem include");
    assert!(priority < include, "priority must come before the include");
    assert!(template.contains("never skipped"), "{template}");
}

#[test]
fn repo_wiki_mem_equals_the_embedded_template() {
    assert_eq!(
        repo_text(REPO_WIKI_MEM_REL),
        embedded_text(WIKI_MEM_TEMPLATE_KEY)
    );
}

#[test]
fn setup_kiro_writes_the_steering_with_the_priority_paragraph() {
    let (_dir, root) = setup_test_project();

    let res = run_cli(&root, &["setup", "kiro"]);

    assert_success!(res);
    let steering = std::fs::read_to_string(root.join(STEERING_OUTPUT_REL)).expect("steering file");
    assert_eq!(steering, embedded_text(STEERING_TEMPLATE_KEY));
    assert!(steering.contains(STEERING_PRIORITY_HEADING), "{steering}");
}
