#[path = "helpers/cli.rs"]
mod helpers;
use helpers::{run_cli, run_cli_with_stdin};

#[path = "helpers/setup.rs"]
mod setup;
use setup::setup_test_project;

#[path = "helpers/macros.rs"]
mod _macros;

#[test]
fn memory_as_wiki_page() {
    let (_dir, root) = setup_test_project();

    let res = run_cli_with_stdin(
        &root,
        &[
            "page",
            "create",
            "memory/e2e-memory",
            "E2E Memory",
            "--page-type",
            "memory",
        ],
        "This is a memory entry for E2E testing with meaningful content.",
    );
    assert_success!(res);

    let res = run_cli(&root, &["page", "list", "--json"]);
    assert_success!(res);
    let parsed: serde_json::Value =
        serde_json::from_str(&res.stdout).expect("valid JSON from page list");
    let total = parsed.get("total").and_then(|v| v.as_u64()).unwrap_or(0);
    assert!(total >= 1, "expected at least 1 page, got {}", total);

    let res = run_cli(&root, &["search", "query", "E2E Memory", "--json"]);
    assert_success!(res);

    let mem_file = root
        .join(".wm")
        .join("wiki")
        .join("memory")
        .join("e2e-memory.md");
    assert!(
        mem_file.exists(),
        "memory page file should exist at {}",
        mem_file.display()
    );
}
