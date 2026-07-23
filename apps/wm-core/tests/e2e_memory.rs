// ─── E2E: Memory Pages ────────────────────────────────────────
// Tests creating and interacting with memory-type wiki pages.

mod helpers;

use helpers::{run_cli, run_cli_with_stdin, setup_test_project};

#[test]
fn memory_as_wiki_page() {
    let (_dir, root) = setup_test_project();

    // Create a memory page with --page-type memory
    let res = run_cli_with_stdin(
        &root,
        &[
            "page", "create", "memory/e2e-memory",
            "E2E Memory",
            "--page-type", "memory",
        ],
        "This is a memory entry for E2E testing with meaningful content.",
    );
    assert_success!(res);

    // List pages with --json and verify memory page is listed
    let res = run_cli(&root, &["page", "list", "--json"]);
    assert_success!(res);
    let parsed: serde_json::Value =
        serde_json::from_str(&res.stdout).expect("valid JSON from page list");
    let total = parsed.get("total").and_then(|v| v.as_u64()).unwrap_or(0);
    assert!(total >= 1, "expected at least 1 page, got {}", total);

    // Search for the memory content
    let res = run_cli(&root, &[
        "search", "query", "E2E Memory", "--json",
    ]);
    assert_success!(res);

    // Verify the .wm/wiki/memory/e2e-memory.md file exists on disk
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
