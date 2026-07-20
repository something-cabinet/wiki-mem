//! IPC command tests — read-only against real wiki, CRUD against temp wiki.
//!
//! Note: The Tauri app's command handlers use wiki_dir() based on current_dir(),
//! and list_pages/read_all_pages do NOT recurse into subdirectories.
//! Pre-existing issues documented in task:verify-tauri-backend-commands.

use crate::pilot_ipc_with_args;
use crate::TestResults;

/// Tauri commands expect args wrapped in {"payload": {...}} when they have
/// a `payload: SomeType` parameter. Commands with no parameters use {}.
fn args(payload: &str) -> String {
    format!("{{\"payload\":{}}}", payload)
}

/// Run read-only IPC assertions against the real project wiki.
pub fn run_readonly_tests(results: &mut TestResults) {
    println!("\n── Read-only IPC tests ──");

    // get_initial — no args
    if let Ok(val) = pilot_ipc_with_args("get_initial", "{}") {
        let nodes = val.get("graph_node_count").and_then(|v| v.as_u64()).unwrap_or(0);
        let edges = val.get("graph_edge_count").and_then(|v| v.as_u64()).unwrap_or(0);
        let success = val.get("success").and_then(|v| v.as_bool()).unwrap_or(false);
        if success && nodes > 0 {
            results.pass(&format!("get_initial — {} nodes, {} edges", nodes, edges));
        } else {
            results.fail("get_initial", &format!("success={}, nodes={}", success, nodes));
        }
    } else {
        results.fail("get_initial", "command failed");
    }

    // search — wrapped in payload
    if let Ok(val) = pilot_ipc_with_args("search", &args(r#"{"q":"spec","type":"page"}"#)) {
        let items = val.get("results").and_then(|v| v.as_array()).map(|a| a.len()).unwrap_or(0);
        if items > 0 {
            results.pass(&format!("search — {} results for 'spec'", items));
        } else {
            results.fail("search", "empty results");
        }
    } else {
        results.fail("search", "command failed");
    }

    // search empty query — returns empty results (expected behavior)
    if let Ok(_val) = pilot_ipc_with_args("search", &args(r#"{"q":"","type":"page"}"#)) {
        results.pass("search — empty query handles gracefully");
    } else {
        results.fail("search-empty", "command failed");
    }

    // list_pages — known pre-existing issue with non-recursive read_dir
    if let Ok(val) = pilot_ipc_with_args("list_pages", "{}") {
        let pages = val.get("pages").and_then(|v| v.as_array()).map(|a| a.len()).unwrap_or(0);
        if pages > 0 {
            results.pass(&format!("list_pages — {} pages", pages));
        } else {
            results.fail("list_pages", "empty (known: non-recursive read_dir)");
        }
    } else {
        results.fail("list_pages", "command failed");
    }

    // get_page for a known page — pre-existing wiki_dir bug
    if let Ok(val) = pilot_ipc_with_args("get_page", &args(r#"{"id":"wiki:specs:tauri-pilot-testing"}"#)) {
        let success = val.get("success").and_then(|v| v.as_bool()).unwrap_or(false);
        if success {
            let title = val.get("title").and_then(|v| v.as_str()).unwrap_or("");
            if title == "Tauri Pilot Testing" {
                results.pass("get_page — known page returns correct title");
            } else {
                results.fail("get_page", &format!("title mismatch: {:?}", title));
            }
        } else {
            // Page not found due to non-recursive read_dir or wiki_dir mismatch
            results.fail("get_page", "not found (known: wiki_dir/directory issue)");
        }
    } else {
        results.fail("get_page", "command failed");
    }

    // get_page — non-existent page
    if let Ok(val) = pilot_ipc_with_args("get_page", &args(r#"{"id":"wiki:nonexistent-page-12345"}"#)) {
        let success = val.get("success").and_then(|v| v.as_bool()).unwrap_or(true);
        if !success {
            results.pass("get_page — non-existent page returns error");
        } else {
            results.fail("get_page-not-found", "expected error");
        }
    } else {
        results.pass("get_page — non-existent page returns error");
    }

    // task_board — no args, may return empty if no wiki data
    if let Ok(val) = pilot_ipc_with_args("task_board", "{}") {
        let columns = val.get("columns").and_then(|v| v.as_object()).map(|o| o.len()).unwrap_or(0);
        let tasks = val.get("tasks").and_then(|v| v.as_array()).map(|a| a.len()).unwrap_or(0);
        if columns > 0 || tasks > 0 {
            results.pass(&format!("task_board — {} columns, {} tasks", columns, tasks));
        } else {
            results.fail("task_board", "empty (known: wiki_dir issue)");
        }
    } else {
        results.fail("task_board", "command failed");
    }

    // get_graph_full — no args
    if let Ok(val) = pilot_ipc_with_args("get_graph_full", "{}") {
        let nodes = val.get("nodes").and_then(|v| v.as_array()).map(|a| a.len()).unwrap_or(0);
        let edges = val.get("edges").and_then(|v| v.as_array()).map(|a| a.len()).unwrap_or(0);
        if nodes > 0 {
            results.pass(&format!("get_graph_full — {} nodes, {} edges", nodes, edges));
        } else {
            results.fail("get_graph_full", "no nodes");
        }
    } else {
        results.fail("get_graph_full", "command failed");
    }

    // get_graph_stats — no args
    if let Ok(val) = pilot_ipc_with_args("get_graph_stats", "{}") {
        let nodes = val.get("node_count").and_then(|v| v.as_u64()).unwrap_or(0);
        let edges = val.get("edge_count").and_then(|v| v.as_u64()).unwrap_or(0);
        if nodes > 0 && edges > 0 {
            results.pass(&format!("get_graph_stats — {} nodes, {} edges", nodes, edges));
        } else {
            results.fail("get_graph_stats", &format!("nodes={}, edges={}", nodes, edges));
        }
    } else {
        results.fail("get_graph_stats", "command failed");
    }

    // get_graph_neighbors — wrapped in payload
    if let Ok(val) = pilot_ipc_with_args("get_graph_neighbors", &args(r#"{"id":"wiki:specs:tauri-pilot-testing"}"#)) {
        let neighbors = val.get("neighbors").and_then(|v| v.as_array()).map(|a| a.len()).unwrap_or(0);
        results.pass(&format!("get_graph_neighbors — {} neighbors", neighbors));
    } else {
        results.fail("get_graph_neighbors", "command failed");
    }

    // compute_layout — wrapped in payload
    if let Ok(val) = pilot_ipc_with_args("compute_layout", &args(r#"{"nodes":[{"id":"a"},{"id":"b"}],"edges":[{"source":0,"target":1}]}"#)) {
        let success = val.get("success").and_then(|v| v.as_bool()).unwrap_or(false);
        let nodes = val.get("nodes").and_then(|v| v.as_u64()).unwrap_or(0);
        if success && nodes == 2 {
            results.pass("compute_layout — success");
        } else {
            results.fail("compute_layout", &format!("success={}, nodes={}", success, nodes));
        }
    } else {
        results.fail("compute_layout", "command failed");
    }

    // get_captured_events — no args
    if let Ok(val) = pilot_ipc_with_args("get_captured_events", "{}") {
        let events = val.get("events").and_then(|v| v.as_array()).map(|a| a.len()).unwrap_or(0);
        if events >= 3 {
            results.pass(&format!("get_captured_events — {} events", events));
        } else {
            results.fail("get_captured_events", &format!("only {} events", events));
        }
    } else {
        results.fail("get_captured_events", "command failed");
    }

    // clear_captured_events — no args
    if let Ok(val) = pilot_ipc_with_args("clear_captured_events", "{}") {
        let success = val.get("success").and_then(|v| v.as_bool()).unwrap_or(false);
        if success {
            results.pass("clear_captured_events — cleared");
        } else {
            results.fail("clear_captured_events", "unexpected response");
        }
    } else {
        results.fail("clear_captured_events", "command failed");
    }

    // list_memory — wrapped in payload
    if let Ok(val) = pilot_ipc_with_args("list_memory", &args("{}")) {
        let entries = val.as_array().map(|a| a.len()).unwrap_or(0);
        results.pass(&format!("list_memory — {} entries", entries));
    } else {
        results.fail("list_memory", "command failed");
    }
}

/// CRUD tests against temp wiki.
pub fn run_crud_tests(results: &mut TestResults) {
    println!("\n── CRUD IPC tests ──");

    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
    let test_id = format!("_test_page_pilot_{}", ts);
    let wiki_id = format!("wiki:{}", test_id);

    // create_page — wrapped in payload
    if let Ok(val) = pilot_ipc_with_args("create_page", &args(&format!(
        r#"{{"id":"{}","title":"Pilot Test Page","type":"concept","content":"Created by pilot-runner"}}"#,
        test_id
    ))) {
        let success = val.get("success").and_then(|v| v.as_bool()).unwrap_or(false);
        if success {
            results.pass("create_page — created");
        } else {
            results.fail("create_page", "success=false");
        }
    } else {
        results.fail("create_page", "command failed");
    }

    // get_page — verify created
    if let Ok(val) = pilot_ipc_with_args("get_page", &args(&format!(r#"{{"id":"{}"}}"#, wiki_id))) {
        let title = val.get("title").and_then(|v| v.as_str()).unwrap_or("");
        if title == "Pilot Test Page" {
            results.pass("get_page — created page has correct title");
        } else {
            results.fail("get_page-created", &format!("title: {:?}", title));
        }
    } else {
        results.fail("get_page-created", "command failed");
    }

    // update_page — wrapped in payload
    if let Ok(val) = pilot_ipc_with_args("update_page", &args(&format!(
        r#"{{"id":"{}","title":"Pilot Test Page Updated"}}"#,
        wiki_id
    ))) {
        let success = val.get("success").and_then(|v| v.as_bool()).unwrap_or(false);
        if success {
            results.pass("update_page — title updated");
        } else {
            results.fail("update_page", "success=false");
        }
    } else {
        results.fail("update_page", "command failed");
    }

    // delete_page — wrapped in payload
    if let Ok(val) = pilot_ipc_with_args("delete_page", &args(&format!(r#"{{"id":"{}"}}"#, wiki_id))) {
        let success = val.get("success").and_then(|v| v.as_bool()).unwrap_or(false);
        if success {
            results.pass("delete_page — deleted");
        } else {
            results.fail("delete_page", "success=false");
        }
    } else {
        results.fail("delete_page", "command failed");
    }
}

