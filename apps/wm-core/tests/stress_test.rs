
mod helpers;

use helpers::{run_cli, setup_test_project};
use std::time::Instant;

#[test]
#[ignore]
fn test_1000_page_graph_rebuild() {
    let (_dir, root) = setup_test_project();

    for i in 0..1000 {
        let res = run_cli(&root, &[
            "page", "create", &format!("concepts/page-{}", i),
            &format!("Page {}", i),
            "--content", &format!("Content for page {} with some searchable text for benchmark purposes.", i),
        ]);
        assert_success!(res);
    }

    let start = Instant::now();
    let res = run_cli(&root, &["index", "rebuild"]);
    let duration = start.elapsed();
    assert_success!(res);
    assert!(duration.as_secs() < 5, "graph rebuild took {:.1}s (expected <5s)", duration.as_secs_f64());

    let res = run_cli(&root, &["graph", "stats", "--json"]);
    assert_success!(res);
}

#[test]
#[ignore]
fn test_version_compaction() {
    let (_dir, root) = setup_test_project();

    let res = run_cli(&root, &[
        "page", "create", "tasks/compact-test",
        "Original Title",
        "--content", "Version compaction test.",
    ]);
    assert_success!(res);

    let page_path = root.join(".wm").join("wiki").join("tasks").join("compact-test.md");
    for i in 0..500 {
        let content = std::fs::read_to_string(&page_path).unwrap_or_default();
        let updated = content.replace(
            &format!("updated {}", (i as i32).saturating_sub(1)),
            &format!("updated {}", i),
        );
        let new_content = if i == 0 {
            content.replace("Original Title", &format!("Updated {}", i))
        } else {
            updated
        };
        std::fs::write(&page_path, new_content).expect("write");
    }

    let res = run_cli(&root, &["index", "rebuild"]);
    assert_success!(res);

    let versions_dir = root.join(".wm").join("versions");
    if versions_dir.exists() {
        let mut total_size = 0u64;
        if let Ok(entries) = std::fs::read_dir(&versions_dir) {
            for entry in entries.flatten() {
                if let Ok(meta) = entry.metadata() {
                    total_size += meta.len();
                }
            }
        }
        assert!(total_size < 100_000, "version files total {total_size} bytes (expected <100KB)");
    }
}
