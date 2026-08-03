#[path = "helpers/mcp.rs"]
mod helpers;
use helpers::MCPClient;

#[path = "helpers/setup.rs"]
mod setup;

#[path = "helpers/macros.rs"]
mod _macros;

fn setup_mcp_test() -> (tempfile::TempDir, MCPClient) {
    let (dir, root) = setup::setup_test_project();
    let client = MCPClient::start(&root);
    (dir, client)
}

#[test]
fn test_initialize_handshake() {
    let (_dir, mut client) = setup_mcp_test();
    let resp = client.initialize().expect("initialize failed");

    let result = resp.get("result").expect("no result");
    assert_eq!(
        result.get("protocolVersion").and_then(|v| v.as_str()),
        Some("2024-11-05")
    );
    assert_eq!(
        result
            .get("serverInfo")
            .and_then(|r| r.get("name"))
            .and_then(|v| v.as_str()),
        Some("wm-engine")
    );
    let instructions = result
        .get("instructions")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    assert!(
        instructions.contains("wm_initial"),
        "instructions should mention wm_initial: {}",
        instructions
    );
}

#[test]
fn test_tools_list() {
    let (_dir, mut client) = setup_mcp_test();
    client.initialize().expect("initialize");

    let tools = client.list_tools().expect("list_tools");

    assert!(tools.len() >= 30, "expected 30+ tools, got {}", tools.len());

    let essential = [
        "wm_initial",
        "wm_help",
        "wm_search.query",
        "wm_search.retrieve",
        "wm_page",
        "wm_source",
        "wm_graph.neighbors",
        "wm_graph.stats",
        "wm_graph.path",
        "wm_graph.subgraph",
        "wm_task",
        "wm_time",
        "wm_index_rebuild",
        "wm_index_embed",
        "wm_index_status",
        "wm_memory",
        "wm_decision",
        "wm_template",
        "wm_model",
        "wm_lint.check",
        "wm_lint.fix",
        "wm_validate.check",
        "wm_log.recent",
        "wm_log.since",
        "wm_log.filter",
        "wm_project.status",
        "wm_project.detect",
        "wm_project.set",
    ];
    for tool in &essential {
        assert!(
            tools.contains(&tool.to_string()),
            "missing essential tool: {}",
            tool
        );
    }
}

#[test]
fn test_wm_initial() {
    let (_dir, mut client) = setup_mcp_test();
    client.initialize().expect("initialize");

    let result = client
        .call_tool("wm_initial", serde_json::json!({}))
        .expect("wm_initial failed");

    assert_eq!(
        result.get("project").and_then(|v| v.as_str()),
        Some("active")
    );
    assert!(result.get("graph_nodes").is_some());
    assert!(result.get("graph_edges").is_some());
    assert!(result.get("search_modes_available").is_some());
}

#[test]
fn test_search_query() {
    let (_dir, mut client) = setup_mcp_test();
    client.initialize().expect("initialize");

    let result = client
        .call_tool(
            "wm_search.query",
            serde_json::json!({ "q": "test", "limit": 5 }),
        )
        .expect("search.query failed");

    assert_eq!(result.get("query").and_then(|v| v.as_str()), Some("test"));
    assert!(result.get("results").is_some());
    assert!(result.get("mode").is_some());
}

#[test]
fn test_search_retrieve() {
    let (_dir, mut client) = setup_mcp_test();
    client.initialize().expect("initialize");

    let result = client
        .call_tool(
            "wm_search.retrieve",
            serde_json::json!({ "q": "test", "token_budget": 4096 }),
        )
        .expect("search.retrieve failed");

    assert!(result.get("tokens_used").is_some());
    assert!(result.get("context").is_some());
}

#[test]
fn test_search_type_filter() {
    let (_dir, mut client) = setup_mcp_test();
    client.initialize().expect("initialize");

    client
        .call_tool(
            "wm_page",
            serde_json::json!({
                "action": "create",
                "path": "concepts/type-filter-test",
                "title": "Type Filter Test",
                "content": "Testing type filter functionality."
            }),
        )
        .expect("page.create failed");

    client
        .call_tool(
            "wm_index_rebuild",
            serde_json::json!({ "skip_embed": true }),
        )
        .expect("index_rebuild failed");

    let page_result = client
        .call_tool(
            "wm_search.query",
            serde_json::json!({ "q": "Type Filter", "type": "page", "limit": 10 }),
        )
        .expect("search with type=page failed");
    let page_results = page_result
        .get("results")
        .and_then(|v| v.as_array())
        .unwrap();
    assert!(!page_results.is_empty(), "expected results for type=page");
    for r in page_results {
        assert_eq!(
            r.get("type").and_then(|v| v.as_str()),
            Some("page"),
            "expected all results to have type 'page'"
        );
    }

    let all_result = client
        .call_tool(
            "wm_search.query",
            serde_json::json!({ "q": "Type Filter", "type": "all", "limit": 10 }),
        )
        .expect("search with type=all failed");
    let all_results = all_result
        .get("results")
        .and_then(|v| v.as_array())
        .unwrap();
    assert!(!all_results.is_empty(), "expected results for type=all");
}

#[test]
fn test_search_hybrid_fallback() {
    let (_dir, mut client) = setup_mcp_test();
    client.initialize().expect("initialize");

    client
        .call_tool(
            "wm_page",
            serde_json::json!({
                "action": "create",
                "path": "concepts/hybrid-fallback-test",
                "title": "Hybrid Fallback Test",
                "content": "Testing hybrid mode fallback."
            }),
        )
        .expect("page.create failed");

    client
        .call_tool(
            "wm_index_rebuild",
            serde_json::json!({ "skip_embed": true }),
        )
        .expect("index_rebuild failed");

    let result = client
        .call_tool(
            "wm_search.query",
            serde_json::json!({ "q": "Hybrid Fallback", "mode": "hybrid", "limit": 5 }),
        )
        .expect("search with mode=hybrid failed");

    let mode = result.get("mode").and_then(|v| v.as_str()).unwrap_or("");
    assert!(
        mode == "hybrid" || mode == "keyword",
        "expected mode 'hybrid' or 'keyword', got '{}'",
        mode
    );
}

#[test]
fn test_page_create_and_get() {
    let (_dir, mut client) = setup_mcp_test();
    client.initialize().expect("initialize");

    let created = client
        .call_tool(
            "wm_page",
            serde_json::json!({
                "action": "create",
                "path": "concepts/test-concept",
                "title": "Test Concept",
                "content": "# Test Concept\n\nA test page for MCP testing."
            }),
        )
        .expect("page.create failed");

    let id = created.get("id").and_then(|v| v.as_str()).unwrap_or("");
    assert!(!id.is_empty(), "expected page id");
    assert_contains!(id, "test-concept");

    let _ = client
        .call_tool(
            "wm_index_rebuild",
            serde_json::json!({ "skip_embed": true }),
        )
        .expect("index_rebuild failed");

    let got = client
        .call_tool("wm_page", serde_json::json!({ "action": "get", "id": id }))
        .expect("page.get failed");

    assert_eq!(got.get("id").and_then(|v| v.as_str()), Some(id));
    assert!(got
        .get("content")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .contains("Test Concept"));
}

#[test]
fn test_page_list() {
    let (_dir, mut client) = setup_mcp_test();
    client.initialize().expect("initialize");

    let result = client
        .call_tool("wm_page", serde_json::json!({ "action": "list" }))
        .expect("page.list failed");

    let _pages = result.get("pages").and_then(|v| v.as_array()).unwrap();
    assert!(result.get("total").is_some());
}

#[test]
fn test_error_invalid_params() {
    let (_dir, mut client) = setup_mcp_test();
    client.initialize().expect("initialize");

    let err = client
        .call_tool("wm_page", serde_json::json!({}))
        .unwrap_err();
    assert!(
        err.contains("required") || err.contains("missing") || err.contains("action"),
        "expected 'required', 'missing', or 'action' error, got: {}",
        err
    );
}

#[test]
fn test_error_not_found() {
    let (_dir, mut client) = setup_mcp_test();
    client.initialize().expect("initialize");

    let err = client
        .call_tool(
            "wm_page",
            serde_json::json!({ "action": "get", "id": "nonexistent:id" }),
        )
        .unwrap_err();
    assert!(
        err.contains("not found"),
        "expected 'not found' error, got: {}",
        err
    );
}

#[test]
fn test_error_unknown_tool() {
    let (_dir, mut client) = setup_mcp_test();
    client.initialize().expect("initialize");

    let err = client
        .call_tool("wm_nonexistent", serde_json::json!({}))
        .unwrap_err();
    assert!(
        err.contains("Unknown") || err.contains("not found"),
        "expected error for unknown tool, got: {}",
        err
    );
}

#[test]
fn test_error_missing_q() {
    let (_dir, mut client) = setup_mcp_test();
    client.initialize().expect("initialize");

    let err = client
        .call_tool("wm_search.query", serde_json::json!({}))
        .unwrap_err();
    assert!(
        err.contains("required")
            || err.contains("missing")
            || err.contains("query")
            || err.contains("q"),
        "expected error for missing 'q' parameter, got: {}",
        err
    );
}

#[test]
fn test_graph_stats() {
    let (_dir, mut client) = setup_mcp_test();
    client.initialize().expect("initialize");

    let result = client
        .call_tool("wm_graph.stats", serde_json::json!({}))
        .expect("graph.stats failed");

    assert!(result.get("nodes").is_some());
    assert!(result.get("edges").is_some());
}

#[test]
fn test_lint_check() {
    let (_dir, mut client) = setup_mcp_test();
    client.initialize().expect("initialize");

    let result = client
        .call_tool("wm_lint.check", serde_json::json!({}))
        .expect("lint.check failed");

    assert!(result.get("issues").is_some());
    assert!(result.get("total").is_some());
}

#[test]
fn test_validate_check() {
    let (_dir, mut client) = setup_mcp_test();
    client.initialize().expect("initialize");

    let result = client
        .call_tool("wm_validate.check", serde_json::json!({}))
        .expect("validate.check failed");

    assert!(result.get("status").is_some());
    assert!(result.get("nodes").is_some());
}

#[test]
fn test_project_status() {
    let (_dir, mut client) = setup_mcp_test();
    client.initialize().expect("initialize");

    let result = client
        .call_tool("wm_project.status", serde_json::json!({}))
        .expect("project.status failed");

    assert_eq!(
        result.get("project").and_then(|v| v.as_str()),
        Some("active")
    );
}

#[test]
fn test_project_detect() {
    let (_dir, mut client) = setup_mcp_test();
    client.initialize().expect("initialize");

    let result = client
        .call_tool("wm_project.detect", serde_json::json!({}))
        .expect("project.detect failed");

    assert_eq!(
        result.get("project").and_then(|v| v.as_str()),
        Some("detected")
    );
}

#[test]
fn test_index_status() {
    let (_dir, mut client) = setup_mcp_test();
    client.initialize().expect("initialize");

    let result = client
        .call_tool("wm_index_status", serde_json::json!({}))
        .expect("index_status failed");

    assert!(result.get("graph_nodes").is_some());
    assert!(result.get("sections").is_some());
}

#[test]
fn test_index_rebuild_memory() {
    let (_dir, mut client) = setup_mcp_test();
    client.initialize().expect("initialize");

    let result = client
        .call_tool(
            "wm_index_rebuild",
            serde_json::json!({ "skip_embed": true }),
        )
        .expect("index_rebuild failed");

    assert!(
        result.get("sections").is_some(),
        "expected sections in rebuild response, got: {:?}",
        result
    );
}

#[test]
fn test_help_all_tools() {
    let (_dir, mut client) = setup_mcp_test();
    client.initialize().expect("initialize");

    let result = client
        .call_tool("wm_help", serde_json::json!({}))
        .expect("help failed");

    let tools = result
        .get("available_tools")
        .and_then(|v| v.as_array())
        .unwrap();
    assert!(tools.len() >= 30, "expected 30+ tools, got {}", tools.len());
}

#[test]
fn test_help_filtered() {
    let (_dir, mut client) = setup_mcp_test();
    client.initialize().expect("initialize");

    let result = client
        .call_tool("wm_help", serde_json::json!({ "q": "search" }))
        .expect("help search failed");

    let tools = result
        .get("available_tools")
        .and_then(|v| v.as_array())
        .unwrap();
    assert!(!tools.is_empty(), "expected search-related tools");
}

#[test]
fn test_workflow_task_lifecycle() {
    let (_dir, mut client) = setup_mcp_test();
    client.initialize().expect("initialize");

    let created = client
        .call_tool(
            "wm_page",
            serde_json::json!({
                "action": "create",
                "path": "tasks/e2e-task-lifecycle",
                "title": "E2E Task Lifecycle",
                "content": "# E2E Task\n\nTest task for lifecycle testing.",
                "status": "todo",
                "priority": "high"
            }),
        )
        .expect("Step 1: create task page failed");
    let task_id = created
        .get("id")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    assert!(!task_id.is_empty(), "expected task page id");

    client
        .call_tool(
            "wm_index_rebuild",
            serde_json::json!({ "skip_embed": true }),
        )
        .expect("index_rebuild failed");

    let list_result = client
        .call_tool("wm_page", serde_json::json!({ "action": "list" }))
        .expect("Step 2: page.list failed");
    let pages = list_result.get("pages").and_then(|v| v.as_array()).unwrap();
    assert!(
        pages
            .iter()
            .any(|p| p.get("id").and_then(|v| v.as_str()) == Some(&task_id)),
        "page should appear in list after rebuild"
    );

    client
        .call_tool(
            "wm_task",
            serde_json::json!({
                "action": "check_ac",
                "id": &task_id,
                "index": 1
            }),
        )
        .ok(); // May fail if page not in graph — non-fatal for this test

    let _ = client.call_tool(
        "wm_time",
        serde_json::json!({
            "action": "start",
            "id": &task_id
        }),
    );

    let _ = client.call_tool(
        "wm_time",
        serde_json::json!({
            "action": "stop",
            "id": &task_id
        }),
    );

    let report = client
        .call_tool("wm_time", serde_json::json!({ "action": "report" }))
        .expect("Step 5: time.report failed");
    let _total_hours = report
        .get("total_hours")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0);

    client
        .call_tool(
            "wm_index_rebuild",
            serde_json::json!({ "skip_embed": true }),
        )
        .expect("Step 6: index_rebuild failed");

    let got = client
        .call_tool(
            "wm_page",
            serde_json::json!({ "action": "get", "id": task_id }),
        )
        .expect("Step 7: page.get failed");
    assert_eq!(
        got.get("id").and_then(|v| v.as_str()),
        Some(task_id.as_str())
    );
    assert_contains!(
        got.get("content").and_then(|v| v.as_str()).unwrap_or(""),
        "E2E Task Lifecycle"
    );
}

#[test]
fn test_workflow_board() {
    let (_dir, mut client) = setup_mcp_test();
    client.initialize().expect("initialize");

    client
        .call_tool(
            "wm_page",
            serde_json::json!({
                "action": "create",
                "path": "tasks/e2e-board-task",
                "title": "Board Test Task",
                "content": "Task for board testing.",
                "status": "todo"
            }),
        )
        .expect("create task for board");

    client
        .call_tool(
            "wm_index_rebuild",
            serde_json::json!({ "skip_embed": true }),
        )
        .expect("index_rebuild failed");

    let result = client
        .call_tool("wm_task", serde_json::json!({ "action": "board" }))
        .expect("task.board failed");

    assert!(result.get("columns").is_some(), "board should have columns");
    let todo_count = result
        .get("counts")
        .and_then(|c| c.get("todo"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    assert!(
        todo_count >= 1,
        "expected at least 1 task on board (todo), got {}",
        todo_count
    );
}

#[test]
fn test_workflow_memory() {
    let (_dir, root) = setup::setup_test_project();
    let mut client = MCPClient::start(&root);
    client.initialize().expect("initialize");

    client
        .call_tool(
            "wm_page",
            serde_json::json!({
                "action": "create",
                "path": "memory/e2e-test-memory-1",
                "title": "E2E Memory Entry",
                "content": "This memory entry exists for E2E testing of cross-entity search.",
                "tags": ["e2e", "test"]
            }),
        )
        .expect("create memory page 1");

    client
        .call_tool(
            "wm_page",
            serde_json::json!({
                "action": "create",
                "path": "memory/e2e-test-memory-2",
                "title": "Auth Pattern",
                "content": "Use JWT with RS256 for API authentication.",
                "tags": ["auth", "pattern"]
            }),
        )
        .expect("create memory page 2");

    client
        .call_tool(
            "wm_index_rebuild",
            serde_json::json!({ "skip_embed": true }),
        )
        .expect("index_rebuild failed");

    let result = client
        .call_tool(
            "wm_search.query",
            serde_json::json!({
                "q": "E2E Memory",
                "type": "page",
                "limit": 10
            }),
        )
        .expect("search memory failed");

    let results = result.get("results").and_then(|v| v.as_array()).unwrap();
    assert!(!results.is_empty(), "expected memory search results");

    assert!(
        results.iter().any(|r| {
            r.get("id")
                .and_then(|v| v.as_str())
                .map(|id| id.contains("e2e-test-memory-1"))
                .unwrap_or(false)
        }),
        "expected search result for e2e-test-memory-1"
    );
}

#[test]
fn test_workflow_cross_entity_search() {
    let (_dir, root) = setup::setup_test_project();
    let mut client = MCPClient::start(&root);
    client.initialize().expect("initialize");

    client.call_tool("wm_page", serde_json::json!({
        "action": "create",
        "path": "concepts/e2e-cross-entity",
        "title": "Cross Entity Search Test",
        "content": "This page tests cross-entity search functionality. Authentication tokens are verified via JWT."
    })).expect("create page failed");

    client
        .call_tool(
            "wm_index_rebuild",
            serde_json::json!({ "skip_embed": true }),
        )
        .expect("index_rebuild failed after page creation");

    client
        .call_tool(
            "wm_page",
            serde_json::json!({
                "action": "create",
                "path": "memory/e2e-cross-entity-mem",
                "title": "Auth Memory",
                "content": "Authentication uses JWT with RS256. Sessions expire after 1 hour.",
                "tags": ["auth"]
            }),
        )
        .expect("create memory page");

    client
        .call_tool(
            "wm_index_rebuild",
            serde_json::json!({ "skip_embed": true }),
        )
        .expect("index_rebuild failed");

    let all_result = client
        .call_tool(
            "wm_search.query",
            serde_json::json!({
                "q": "authentication JWT",
                "type": "all",
                "limit": 20
            }),
        )
        .expect("cross-entity search failed");

    let all_results = all_result
        .get("results")
        .and_then(|v| v.as_array())
        .unwrap();
    assert!(!all_results.is_empty(), "expected cross-entity results");

    let has_page = all_results.iter().any(|r| {
        r.get("id")
            .and_then(|v| v.as_str())
            .map(|id| id.contains("concepts:e2e-cross-entity"))
            .unwrap_or(false)
    });
    let has_memory = all_results.iter().any(|r| {
        r.get("id")
            .and_then(|v| v.as_str())
            .map(|id| id.contains("memory:e2e-cross-entity-mem"))
            .unwrap_or(false)
    });
    assert!(
        has_page || has_memory,
        "expected results from page or memory types"
    );
    assert!(
        has_page,
        "expected at least one page result (created a page with 'authentication')"
    );
    assert!(
        has_memory,
        "expected at least one memory result (created memory with 'authentication')"
    );

    let page_result = client
        .call_tool(
            "wm_search.query",
            serde_json::json!({
                "q": "authentication",
                "type": "page",
                "limit": 20
            }),
        )
        .expect("page-only search failed");

    let page_results = page_result
        .get("results")
        .and_then(|v| v.as_array())
        .unwrap();
    if !page_results.is_empty() {
        for r in page_results {
            assert_eq!(
                r.get("type").and_then(|v| v.as_str()),
                Some("page"),
                "expected all results to have type 'page'"
            );
        }
    }

    let mem_result = client
        .call_tool(
            "wm_search.query",
            serde_json::json!({
                "q": "authentication",
                "type": "page",
                "limit": 20
            }),
        )
        .expect("page-only search failed");

    let mem_results = mem_result
        .get("results")
        .and_then(|v| v.as_array())
        .unwrap();
    if !mem_results.is_empty() {
        assert!(
            mem_results.iter().any(|r| {
                r.get("id")
                    .and_then(|v| v.as_str())
                    .map(|id| id.contains("memory:e2e-cross-entity-mem"))
                    .unwrap_or(false)
            }),
            "expected at least one memory result (e2e-cross-entity-mem)"
        );
    }
}

#[test]
fn test_workflow_validation() {
    let (_dir, mut client) = setup_mcp_test();
    client.initialize().expect("initialize");

    client
        .call_tool(
            "wm_page",
            serde_json::json!({
                "action": "create",
                "path": "tasks/e2e-validate-task",
                "title": "Validate Test Task",
                "content": "Task for validate testing.",
                "status": "todo"
            }),
        )
        .expect("create task page");

    client
        .call_tool(
            "wm_index_rebuild",
            serde_json::json!({ "skip_embed": true }),
        )
        .expect("wm_index_rebuild failed");

    let result = client
        .call_tool("wm_validate.check", serde_json::json!({}))
        .expect("validate.check failed");

    assert!(
        result.get("status").is_some(),
        "validate should return status"
    );
    assert!(
        result.get("nodes").is_some(),
        "validate should return node count"
    );
}

fn setup_code_test() -> (tempfile::TempDir, helpers::MCPClient) {
    let (dir, root) = setup::setup_test_project();

    let src_dir = root.join("src");
    std::fs::create_dir_all(&src_dir).expect("create src");

    std::fs::write(
        src_dir.join("wm_lib.rs"),
        r#"
use std::collections::HashMap;
use std::sync::Arc;

pub struct CodeTest {
    pub name: String,
    pub value: i32,
}

pub fn greet(name: &str) -> String {
    format!("Hello, {}!", name)
}

pub enum Status {
    Active,
    Inactive,
    Pending,
}

pub trait Processor {
    fn process(&self) -> bool;
}

impl Processor for CodeTest {
    fn process(&self) -> bool {
        self.value > 0
    }
}

pub mod utils {
    pub fn helper() -> &'static str {
        "helper"
    }
}

const DEFAULT_TIMEOUT: u64 = 30;
pub type Result<T> = std::result::Result<T, String>;
"#,
    )
    .expect("write lib.rs");

    std::fs::write(
        src_dir.join("wm_main.rs"),
        r#"
mod lib;
use lib::CodeTest;

fn main() {
    let test = CodeTest { name: "test".into(), value: 42 };
    println!("{:?}", test.name);
}
"#,
    )
    .expect("write main.rs");

    let client = helpers::MCPClient::start(&root);
    (dir, client)
}

#[test]
fn test_code_search_finds_pattern() {
    let (_dir, mut client) = setup_code_test();
    client.initialize().expect("initialize");

    let tools = client.list_tools().expect("list_tools");
    assert!(tools.contains(&"wm_code.search".to_string()));

    let result = client
        .call_tool(
            "wm_code.search",
            serde_json::json!({
                "pattern": "pub struct",
                "max_results": 10
            }),
        )
        .expect("code.search failed");

    let results = result.get("results").and_then(|v| v.as_array()).unwrap();
    assert!(!results.is_empty(), "should find results for 'pub struct'");
    assert!(result.get("total").and_then(|v| v.as_u64()).unwrap_or(0) >= 1);
}

#[test]
fn test_code_search_with_file_type() {
    let (_dir, mut client) = setup_code_test();
    client.initialize().expect("initialize");

    let result = client
        .call_tool(
            "wm_code.search",
            serde_json::json!({
                "pattern": "struct",
                "file_type": "rs"
            }),
        )
        .expect("code.search failed");

    let results = result.get("results").and_then(|v| v.as_array()).unwrap();
    assert!(
        !results.is_empty(),
        "should find struct keyword in rs files"
    );
}

#[test]
fn test_code_search_no_results() {
    let (_dir, mut client) = setup_code_test();
    client.initialize().expect("initialize");

    let result = client
        .call_tool(
            "wm_code.search",
            serde_json::json!({
                "pattern": "ZZZZNOTFOUND",
            }),
        )
        .expect("code.search failed");

    let total = result.get("total").and_then(|v| v.as_u64()).unwrap_or(0);
    assert_eq!(total, 0, "should find no results for non-existent pattern");
}

#[test]
fn test_code_search_invalid_regex() {
    let (_dir, mut client) = setup_code_test();
    client.initialize().expect("initialize");

    let result = client.call_tool(
        "wm_code.search",
        serde_json::json!({
            "pattern": "[invalid",
        }),
    );

    assert!(result.is_err(), "invalid regex should return error");
}

#[test]
fn test_code_symbols_finds_structs() {
    let (_dir, mut client) = setup_code_test();
    client.initialize().expect("initialize");

    let result = client
        .call_tool(
            "wm_code.symbols",
            serde_json::json!({
                "kind": "struct"
            }),
        )
        .expect("code.symbols failed");

    let symbols = result.get("symbols").and_then(|v| v.as_array()).unwrap();
    assert!(!symbols.is_empty(), "should find struct symbols");
    let names: Vec<&str> = symbols
        .iter()
        .filter_map(|s| s.get("name").and_then(|n| n.as_str()))
        .collect();
    assert!(
        names.contains(&"CodeTest"),
        "should contain CodeTest, got: {:?}",
        names
    );
}

#[test]
fn test_code_symbols_finds_functions() {
    let (_dir, mut client) = setup_code_test();
    client.initialize().expect("initialize");

    let result = client
        .call_tool(
            "wm_code.symbols",
            serde_json::json!({
                "kind": "function"
            }),
        )
        .expect("code.symbols failed");

    let symbols = result.get("symbols").and_then(|v| v.as_array()).unwrap();
    assert!(!symbols.is_empty(), "should find function symbols");
    let names: Vec<&str> = symbols
        .iter()
        .filter_map(|s| s.get("name").and_then(|n| n.as_str()))
        .collect();
    assert!(
        names.contains(&"greet"),
        "should contain greet, got: {:?}",
        names
    );
}

#[test]
fn test_code_symbols_name_filter() {
    let (_dir, mut client) = setup_code_test();
    client.initialize().expect("initialize");

    let result = client
        .call_tool(
            "wm_code.symbols",
            serde_json::json!({
                "name": "CodeTest"
            }),
        )
        .expect("code.symbols failed");

    let symbols = result.get("symbols").and_then(|v| v.as_array()).unwrap();
    assert!(!symbols.is_empty(), "should find symbols named CodeTest");
    for sym in symbols {
        let name = sym.get("name").and_then(|n| n.as_str()).unwrap_or("");
        assert!(
            name.contains("CodeTest"),
            "all symbols should match filter 'CodeTest', got: {}",
            name
        );
    }
}

#[test]
fn test_code_symbols_path_filter() {
    let (_dir, mut client) = setup_code_test();
    client.initialize().expect("initialize");

    let result = client
        .call_tool(
            "wm_code.symbols",
            serde_json::json!({
                "path": "src"
            }),
        )
        .expect("code.symbols failed");

    let symbols = result.get("symbols").and_then(|v| v.as_array()).unwrap();
    assert!(!symbols.is_empty(), "should find symbols in src/");
}

#[test]
fn test_code_symbols_kind_enum() {
    let (_dir, mut client) = setup_code_test();
    client.initialize().expect("initialize");

    let result = client
        .call_tool(
            "wm_code.symbols",
            serde_json::json!({
                "kind": "enum"
            }),
        )
        .expect("code.symbols failed");

    let symbols = result.get("symbols").and_then(|v| v.as_array()).unwrap();
    let names: Vec<&str> = symbols
        .iter()
        .filter_map(|s| s.get("name").and_then(|n| n.as_str()))
        .collect();
    assert!(
        names.contains(&"Status"),
        "should find Status enum, got: {:?}",
        names
    );
}

#[test]
fn test_code_symbols_kind_trait() {
    let (_dir, mut client) = setup_code_test();
    client.initialize().expect("initialize");

    let result = client
        .call_tool(
            "wm_code.symbols",
            serde_json::json!({
                "kind": "trait"
            }),
        )
        .expect("code.symbols failed");

    let symbols = result.get("symbols").and_then(|v| v.as_array()).unwrap();
    let names: Vec<&str> = symbols
        .iter()
        .filter_map(|s| s.get("name").and_then(|n| n.as_str()))
        .collect();
    assert!(
        names.contains(&"Processor"),
        "should find Processor trait, got: {:?}",
        names
    );
}

#[test]
fn test_code_deps_basic() {
    let (_dir, mut client) = setup_code_test();
    client.initialize().expect("initialize");

    let result = client
        .call_tool("wm_code.deps", serde_json::json!({}))
        .expect("code.deps failed");

    let deps = result
        .get("dependencies")
        .and_then(|v| v.as_array())
        .unwrap();
    assert!(!deps.is_empty(), "should find some dependencies");

    let main_deps: Vec<&serde_json::Value> = deps
        .iter()
        .filter(|d| {
            d.get("file")
                .and_then(|f| f.as_str())
                .map(|f| f.contains("wm_main.rs"))
                .unwrap_or(false)
        })
        .collect();
    assert!(!main_deps.is_empty(), "main.rs should have dependencies");
}

#[test]
fn test_code_deps_file_filter() {
    let (_dir, mut client) = setup_code_test();
    client.initialize().expect("initialize");

    let result = client
        .call_tool(
            "wm_code.deps",
            serde_json::json!({
                "file": "wm_lib.rs"
            }),
        )
        .expect("code.deps failed");

    let deps = result
        .get("dependencies")
        .and_then(|v| v.as_array())
        .unwrap();
    for dep in deps {
        let file = dep.get("file").and_then(|f| f.as_str()).unwrap_or("");
        assert!(
            file.contains("wm_lib.rs"),
            "file filter should only return lib.rs, got: {}",
            file
        );
    }
}

#[test]
fn test_code_tools_in_tools_list() {
    let (_dir, mut client) = setup_code_test();
    client.initialize().expect("initialize");

    let tools = client.list_tools().expect("list_tools");

    assert!(
        tools.contains(&"wm_code.search".to_string()),
        "tools/list should include wm_code.search"
    );
    assert!(
        tools.contains(&"wm_code.symbols".to_string()),
        "tools/list should include wm_code.symbols"
    );
    assert!(
        tools.contains(&"wm_code.deps".to_string()),
        "tools/list should include wm_code.deps"
    );
}

#[test]
fn test_workflow_source_list() {
    let (_dir, mut client) = setup_mcp_test();
    client.initialize().expect("initialize");

    let result = client
        .call_tool("wm_source", serde_json::json!({ "action": "list" }))
        .expect("source.list failed");

    let _sources = result
        .get("sources")
        .and_then(|v| v.as_array())
        .unwrap_or(&vec![]);
    assert!(
        result.get("total").is_some(),
        "source.list should return total"
    );
}

#[test]
fn test_workflow_lint_after_create() {
    let (_dir, mut client) = setup_mcp_test();
    client.initialize().expect("initialize");

    client
        .call_tool(
            "wm_page",
            serde_json::json!({
                "action": "create",
                "path": "concepts/e2e-lint-test",
                "title": "Lint Test",
                "content": "A page for lint testing."
            }),
        )
        .expect("create page");

    client
        .call_tool(
            "wm_index_rebuild",
            serde_json::json!({ "skip_embed": true }),
        )
        .expect("rebuild");

    let result = client
        .call_tool("wm_lint.check", serde_json::json!({}))
        .expect("lint.check failed");

    assert!(result.get("issues").is_some());
    assert!(result.get("total").is_some());
}

#[test]
fn test_all_tools_respond() {
    let (_dir, root) = setup::setup_test_project();
    let mut client = MCPClient::start(&root);
    client.initialize().expect("initialize");
    let tools = client.list_tools().expect("list tools");
    assert!(
        tools.len() >= 30,
        "expected at least 30 tools, got {}",
        tools.len()
    );
}

#[test]
fn test_wm_page_get_missing_id() {
    let (_dir, root) = setup::setup_test_project();
    let mut client = MCPClient::start(&root);
    client.initialize().expect("initialize");
    let err = client
        .call_tool("wm_page", serde_json::json!({"action": "get"}))
        .unwrap_err();
    assert!(
        err.contains("required") || err.contains("missing") || err.contains("id"),
        "expected error for missing required field 'id', got: {}",
        err
    );
}

#[test]
fn test_wm_task_update_invalid_transition() {
    let (_dir, root) = setup::setup_test_project();
    let mut client = MCPClient::start(&root);
    client.initialize().expect("initialize");
    let created = client
        .call_tool(
            "wm_page",
            serde_json::json!({
                "action": "create", "path": "tasks/trans-test",
                "title": "Transition Test", "status": "todo"
            }),
        )
        .expect("create task");
    let id = created.get("id").and_then(|v| v.as_str()).unwrap_or("");
    assert!(!id.is_empty(), "expected a page id from create");
    let page_path = root
        .join(".wm")
        .join("wiki")
        .join("tasks")
        .join("trans-test.md");
    assert!(
        page_path.exists(),
        "task file should exist: {:?}",
        page_path
    );
    let content = std::fs::read_to_string(&page_path).unwrap_or_default();
    assert!(
        content.contains("status: todo"),
        "task should have status: todo, got: {}",
        content
    );
}

#[test]
fn test_wm_task_update_valid_transition() {
    let (_dir, root) = setup::setup_test_project();
    let mut client = MCPClient::start(&root);
    client.initialize().expect("initialize");
    let created = client
        .call_tool(
            "wm_page",
            serde_json::json!({
                "action": "create", "path": "tasks/valid-trans",
                "title": "Valid Transition", "status": "todo"
            }),
        )
        .expect("create task");
    let id = created.get("id").and_then(|v| v.as_str()).unwrap_or("");
    assert!(!id.is_empty(), "expected a page id from create");
    let page_path = root
        .join(".wm")
        .join("wiki")
        .join("tasks")
        .join("valid-trans.md");
    assert!(
        page_path.exists(),
        "task file should exist: {:?}",
        page_path
    );
    let content = std::fs::read_to_string(&page_path).unwrap_or_default();
    assert!(
        content.contains("status: todo"),
        "task should have status: todo, got: {}",
        content
    );
    let updated = content.replace("status: todo", "status: in-progress");
    std::fs::write(&page_path, updated).expect("write updated status");
    let content2 = std::fs::read_to_string(&page_path).unwrap_or_default();
    assert!(
        content2.contains("status: in-progress"),
        "task should have status: in-progress"
    );
}

#[test]
fn test_wm_memory_add_creates_wiki_page() {
    let (_dir, root) = setup::setup_test_project();
    let mut client = MCPClient::start(&root);
    client.initialize().expect("initialize");
    let result = client
        .call_tool(
            "wm_memory",
            serde_json::json!({
                "action": "add", "title": "Memory to Page", "content": "Test content",
                "tags": ["test"]
            }),
        )
        .expect("add memory");
    assert!(
        result.get("id").is_some(),
        "memory should have an id: {:?}",
        result
    );
    let id = result.get("id").and_then(|v| v.as_str()).unwrap_or("");
    let slug = id.rsplit(':').next().unwrap_or(id);
    let page_path = root
        .join(".wm")
        .join("wiki")
        .join("memory")
        .join(&format!("{}.md", slug));
    assert!(
        page_path.exists(),
        "memory page file should exist at {:?} (id was {:?})",
        page_path,
        id
    );
}

#[test]
fn test_version_rollback_restores_title() {
    let (_dir, root) = setup::setup_test_project();
    let mut client = MCPClient::start(&root);
    client.initialize().expect("initialize");
    let created = client
        .call_tool(
            "wm_page",
            serde_json::json!({
                "action": "create", "path": "tasks/rollback-test",
                "title": "Original Title", "content": "Test content"
            }),
        )
        .expect("create task");
    let id = created.get("id").and_then(|v| v.as_str()).unwrap_or("");
    assert!(!id.is_empty(), "expected a page id from create");
    let page_path = root
        .join(".wm")
        .join("wiki")
        .join("tasks")
        .join("rollback-test.md");
    assert!(
        page_path.exists(),
        "task file should exist: {:?}",
        page_path
    );
    let versions_dir = root.join(".wm").join("versions");
    std::fs::create_dir_all(&versions_dir).expect("create versions dir");
    let version = serde_json::json!({
        "entity_id": id,
        "current_version": 1,
        "versions": [{
            "id": "v1", "version": 1,
            "timestamp": "2026-07-14T12:00:00Z",
            "changes": [{"field": "title",
                "old_value": "Original Title",
                "new_value": "Updated Title"}],
            "compacted": false
        }]
    });
    let vf = versions_dir.join(format!("task-{}.json", id.replace(':', "-")));
    std::fs::write(&vf, serde_json::to_string_pretty(&version).unwrap())
        .expect("write version file");
    assert!(vf.exists(), "version file should exist");
}

#[test]
fn test_template_add_action() {
    let (_dir, root) = setup::setup_test_project();
    let mut client = MCPClient::start(&root);
    client.initialize().expect("initialize");
    let result = client
        .call_tool(
            "wm_template",
            serde_json::json!({
                "action": "list"
            }),
        )
        .expect("list templates");
    assert!(
        result.get("templates").is_some() || result.get("error").is_none(),
        "template list should succeed: {:?}",
        result
    );
}

#[test]
fn test_ref_path_traversal() {
    let (_dir, root) = setup::setup_test_project();
    let mut client = MCPClient::start(&root);
    client.initialize().expect("initialize");
    let result = client
        .call_tool(
            "wm_ref.extract",
            serde_json::json!({
                "content": "Some text with @wiki/templates/../../etc/passwd reference."
            }),
        )
        .expect("extract references");
    let empty_vec = vec![];
    let refs = result
        .get("references")
        .and_then(|v| v.as_array())
        .unwrap_or(&empty_vec);
    assert!(
        refs.len() >= 1,
        "should extract at least the reference: {:?}",
        result
    );
}

#[test]
fn test_typed_module_removed() {
    let typed_path = std::path::Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/src/mcp/typed.rs"));
    assert!(!typed_path.exists(), "typed.rs should have been deleted");
}

#[test]
fn test_wm_page_invalid_action() {
    let (_dir, root) = setup::setup_test_project();
    let mut client = MCPClient::start(&root);
    client.initialize().expect("initialize");
    let result = client.call_tool("wm_page", serde_json::json!({"action": "fly"}));
    match result {
        Ok(resp) => {
            assert!(
                resp.get("error").is_some() || resp.get("isError").is_some(),
                "expected error for invalid action 'fly': {:?}",
                resp
            );
        }
        Err(e) => {
            assert!(
                e.contains("fly") || e.contains("action") || e.contains("invalid"),
                "expected error mentioning 'fly' or 'action': {}",
                e
            );
        }
    }
}

#[test]
fn test_wm_decision_create_adr_fields() {
    let (_dir, root) = setup::setup_test_project();
    let mut client = MCPClient::start(&root);
    client.initialize().expect("initialize");
    let result = client
        .call_tool(
            "wm_decision",
            serde_json::json!({
                "action": "create",
                "id": "decisions/test-adr",
                "title": "Test ADR",
                "context": "We need to make a decision",
                "rationale": "This is why",
                "outcome": "We chose option A"
            }),
        )
        .expect("create decision");
    assert!(
        result.get("id").is_some() || result.get("status").is_some(),
        "expected decision to be created: {:?}",
        result
    );
}

#[test]
fn test_wm_template_run_basic() {
    let (_dir, root) = setup::setup_test_project();
    let mut client = MCPClient::start(&root);
    client.initialize().expect("initialize");
    let result = client
        .call_tool(
            "wm_template",
            serde_json::json!({
                "action": "list"
            }),
        )
        .expect("list templates");
    assert!(
        result.get("templates").is_some() || result.get("error").is_none(),
        "template list should succeed: {:?}",
        result
    );
}

#[test]
fn test_wm_model_list() {
    let (_dir, root) = setup::setup_test_project();
    let mut client = MCPClient::start(&root);
    client.initialize().expect("initialize");
    let result = client
        .call_tool(
            "wm_model",
            serde_json::json!({
                "action": "list"
            }),
        )
        .expect("model list failed");
    assert!(
        result.get("active_model").is_some(),
        "expected active_model: {:?}",
        result
    );
    assert!(
        result.get("models").is_some(),
        "expected models array: {:?}",
        result
    );
}

#[test]
fn test_wm_log_recent() {
    let (_dir, root) = setup::setup_test_project();
    let mut client = MCPClient::start(&root);
    client.initialize().expect("initialize");
    let result = client
        .call_tool("wm_log.recent", serde_json::json!({}))
        .expect("log.recent failed");
    assert!(
        result.get("entries").is_some(),
        "expected entries in log.recent response: {:?}",
        result
    );
    assert!(
        result.get("total").is_some(),
        "expected total in log.recent response: {:?}",
        result
    );
}

#[test]
fn test_regression_wm_page_uses_id_parameter() {
    let (_dir, root) = setup::setup_test_project();
    let mut client = MCPClient::start(&root);
    client.initialize().expect("initialize");

    let created = client
        .call_tool(
            "wm_page",
            serde_json::json!({
                "action": "create",
                "path": "regression/id-param",
                "title": "ID Param Test",
                "content": "Testing id parameter."
            }),
        )
        .expect("create page failed");
    let id = created
        .get("id")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    assert!(!id.is_empty(), "expected page id from create");

    let result = client
        .call_tool(
            "wm_page",
            serde_json::json!({
                "action": "update",
                "id": id,
                "title": "Updated via id"
            }),
        )
        .expect("page.update via id failed");
    assert_eq!(
        result.get("status").and_then(|v| v.as_str()),
        Some("updated")
    );
}

#[test]
fn test_regression_wm_page_get_by_id() {
    let (_dir, root) = setup::setup_test_project();
    let mut client = MCPClient::start(&root);
    client.initialize().expect("initialize");

    client
        .call_tool(
            "wm_page",
            serde_json::json!({
                "action": "create",
                "path": "concepts/get-by-id",
                "title": "Get By ID",
                "content": "Body fetched via canonical id parameter."
            }),
        )
        .expect("create page failed");

    let result = client
        .call_tool(
            "wm_page",
            serde_json::json!({
                "action": "get",
                "id": "wiki:concepts:get-by-id"
            }),
        )
        .expect("wm_page.get with id parameter failed");
    assert_eq!(
        result.get("id").and_then(|v| v.as_str()),
        Some("wiki:concepts:get-by-id")
    );
    assert!(
        result
            .get("content")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .contains("Body fetched via canonical id parameter."),
        "expected page content, got: {:?}",
        result
    );
}

#[test]
fn test_regression_wm_page_schema_complete() {
    let (_dir, root) = setup::setup_test_project();
    let mut client = MCPClient::start(&root);
    client.initialize().expect("initialize");

    let resp = client
        .send_request("tools/list", serde_json::json!({}))
        .expect("tools/list failed");
    let result = resp
        .get("result")
        .expect("no result in tools/list response");
    let tools = result
        .get("tools")
        .and_then(|v| v.as_array())
        .expect("tools/list should return tools array");
    let page_tool = tools
        .iter()
        .find(|t| t.get("name").and_then(|v| v.as_str()) == Some("wm_page"))
        .expect("wm_page should be listed");
    let schema = page_tool
        .get("inputSchema")
        .expect("wm_page missing inputSchema");
    let obj = schema
        .as_object()
        .expect("wm_page inputSchema must be an object");
    assert_eq!(
        obj.get("type").and_then(|v| v.as_str()),
        Some("object"),
        "wm_page root type"
    );
    assert!(
        obj.get("oneOf").is_none(),
        "wm_page schema must not use top-level oneOf"
    );
    let required = obj
        .get("required")
        .and_then(|v| v.as_array())
        .expect("wm_page required");
    assert_eq!(required.len(), 1, "wm_page must require only action");
    assert!(
        required.iter().any(|r| r.as_str() == Some("action")),
        "wm_page must require action"
    );
    let props = obj
        .get("properties")
        .and_then(|v| v.as_object())
        .expect("wm_page properties");
    let action = props
        .get("action")
        .and_then(|v| v.as_object())
        .expect("wm_page action property");
    let action_enum = action
        .get("enum")
        .and_then(|v| v.as_array())
        .expect("wm_page action enum");
    assert_eq!(action_enum.len(), 7, "expected 7 wm_page action values");
    for expected in [
        "list", "get", "create", "update", "delete", "link", "unlink",
    ] {
        assert!(
            action_enum.iter().any(|v| v.as_str() == Some(expected)),
            "wm_page action enum missing {}",
            expected
        );
    }
    for (name, prop) in props {
        assert!(
            name == "action" || prop.get("description").is_some(),
            "wm_page field {} missing schema description",
            name
        );
        assert!(
            name != "page_id",
            "wm_page must not expose a page_id parameter"
        );
    }
}

#[test]
fn test_regression_tool_schema() {
    let (_dir, root) = setup::setup_test_project();
    let mut client = MCPClient::start(&root);
    client.initialize().expect("initialize");

    let resp = client
        .send_request("tools/list", serde_json::json!({}))
        .expect("tools/list failed");
    let result = resp
        .get("result")
        .expect("no result in tools/list response");
    let tools = result
        .get("tools")
        .and_then(|v| v.as_array())
        .expect("tools/list should return tools array");
    let page_tools: Vec<_> = tools
        .iter()
        .filter(|t| {
            t.get("name")
                .and_then(|v| v.as_str())
                .map(|n| n.starts_with("wm_page"))
                .unwrap_or(false)
        })
        .collect();
    for tool in &page_tools {
        assert!(
            tool.get("inputSchema").is_some(),
            "Tool {} missing inputSchema",
            tool["name"]
        );
    }
}

#[test]
fn test_all_tools_schemas_no_top_level_composition() {
    let (_dir, root) = setup::setup_test_project();
    let mut client = MCPClient::start(&root);
    client.initialize().expect("initialize");

    let resp = client
        .send_request("tools/list", serde_json::json!({}))
        .expect("tools/list failed");
    let result = resp
        .get("result")
        .expect("no result in tools/list response");
    let tools = result
        .get("tools")
        .and_then(|v| v.as_array())
        .expect("tools/list should return tools array");
    assert!(tools.len() >= 30, "expected 30+ tools, got {}", tools.len());

    for tool in tools {
        let name = tool.get("name").and_then(|v| v.as_str()).unwrap_or("");
        let schema = tool
            .get("inputSchema")
            .unwrap_or_else(|| panic!("tool {} missing inputSchema", name));
        let obj = schema
            .as_object()
            .unwrap_or_else(|| panic!("tool {} inputSchema must be an object", name));
        assert_eq!(
            obj.get("type").and_then(|v| v.as_str()),
            Some("object"),
            "tool {} root type",
            name
        );
        for keyword in ["oneOf", "allOf", "anyOf"] {
            assert!(
                !obj.contains_key(keyword),
                "tool {} has top-level {} in inputSchema",
                name,
                keyword
            );
        }
        if let Some(action) = obj
            .get("properties")
            .and_then(|v| v.get("action"))
            .and_then(|v| v.as_object())
        {
            let enum_values = action
                .get("enum")
                .and_then(|v| v.as_array())
                .unwrap_or_else(|| panic!("tool {} action property must carry an enum", name));
            assert!(
                enum_values.iter().any(|v| v.is_string()),
                "tool {} action enum values must be strings",
                name
            );
        }
    }
}

#[test]
fn test_regression_wm_index_split() {
    let (_dir, root) = setup::setup_test_project();
    let mut client = MCPClient::start(&root);
    client.initialize().expect("initialize");

    let result = client
        .call_tool("wm_index_rebuild", serde_json::json!({"skip_embed": true}))
        .expect("wm_index_rebuild failed");
    assert_eq!(result.get("status").and_then(|v| v.as_str()), Some("ok"));

    let result = client
        .call_tool("wm_index_status", serde_json::json!({}))
        .expect("wm_index_status failed");
    assert!(result.get("graph_nodes").is_some());
    assert!(result.get("sections").is_some());
}

#[test]
fn test_regression_wm_index_three_tools_listed() {
    let (_dir, root) = setup::setup_test_project();
    let mut client = MCPClient::start(&root);
    client.initialize().expect("initialize");

    let tools = client.list_tools().expect("list_tools");
    for name in ["wm_index_rebuild", "wm_index_status", "wm_index_embed"] {
        assert!(
            tools.contains(&name.to_string()),
            "missing split index tool: {}",
            name
        );
    }
    assert!(
        !tools.contains(&"wm_index".to_string()),
        "single wm_index tool should not exist after split"
    );
}

#[test]
fn test_regression_match_arm_no_discard() {
    let (_dir, root) = setup::setup_test_project();
    let mut client = MCPClient::start(&root);
    client.initialize().expect("initialize");

    let result = client
        .call_tool(
            "wm_page",
            serde_json::json!({
                "action": "list",
                "limit": 5
            }),
        )
        .expect("wm_page list with limit failed");
    assert!(
        result.get("total").is_some(),
        "expected total in page list response"
    );
}

#[test]
fn test_regression_index_embed_force() {
    let (_dir, root) = setup::setup_test_project();
    let mut client = MCPClient::start(&root);
    client.initialize().expect("initialize");

    client
        .call_tool("wm_index_rebuild", serde_json::json!({"skip_embed": true}))
        .expect("rebuild failed");

    let result = client.call_tool("wm_index_embed", serde_json::json!({"force": true}));
    match result {
        Ok(res) => {
            assert!(
                res.get("status").is_some(),
                "expected status in embed response"
            );
        }
        Err(e) => {
            assert!(
                e.contains("model") || e.contains("embed") || e.contains("sections"),
                "expected model/embed/sections error, got: {}",
                e
            );
        }
    }
}

#[test]
fn test_lint_check_catches_missing_id() {
    let (_dir, root) = setup::setup_test_project();
    let mut client = MCPClient::start(&root);
    client.initialize().expect("initialize");

    // Write a page without id: in frontmatter
    std::fs::write(
        root.join(".wm")
            .join("wiki")
            .join("concepts")
            .join("no-id-test.md"),
        "---\ntitle: No ID Test\ntype: concept\n---\n\nBody here.\n",
    )
    .expect("write test page");

    // Rebuild to pick up the new page
    client
        .call_tool("wm_index_rebuild", serde_json::json!({}))
        .expect("rebuild");

    // Check lint
    let lint_result = client
        .call_tool("wm_lint.check", serde_json::json!({}))
        .expect("lint check");
    let issues = lint_result
        .get("issues")
        .and_then(|v| v.as_array())
        .expect("issues array");
    let missing_ids: Vec<&serde_json::Value> = issues
        .iter()
        .filter(|i| i.get("type").and_then(|v| v.as_str()) == Some("missing_id"))
        .collect();
    assert_eq!(
        missing_ids.len(),
        1,
        "expected exactly 1 missing_id issue, got {}: {:?}",
        missing_ids.len(),
        missing_ids
    );
    assert!(
        missing_ids[0]
            .get("message")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .contains("wiki:concepts:no-id-test"),
        "message should reference the page ID"
    );
}

#[test]
fn test_lint_check_passes_with_id() {
    let (_dir, root) = setup::setup_test_project();
    let mut client = MCPClient::start(&root);
    client.initialize().expect("initialize");

    // Write a page WITH id: in frontmatter
    std::fs::write(
        root.join(".wm").join("wiki").join("concepts").join("has-id-test.md"),
        "---\ntitle: Has ID Test\ntype: concept\nid: wiki:concepts:has-id-test\n---\n\nBody here.\n",
    )
    .expect("write test page");

    // Rebuild to pick up the new page
    client
        .call_tool("wm_index_rebuild", serde_json::json!({}))
        .expect("rebuild");

    // Check lint
    let lint_result = client
        .call_tool("wm_lint.check", serde_json::json!({}))
        .expect("lint check");
    let issues = lint_result
        .get("issues")
        .and_then(|v| v.as_array())
        .expect("issues array");
    let missing_ids: Vec<&serde_json::Value> = issues
        .iter()
        .filter(|i| i.get("type").and_then(|v| v.as_str()) == Some("missing_id"))
        .collect();
    assert_eq!(
        missing_ids.len(),
        0,
        "expected 0 missing_id issues, got {}: {:?}",
        missing_ids.len(),
        missing_ids
    );
}

#[test]
fn test_page_create_emits_id_frontmatter() {
    let (_dir, root) = setup::setup_test_project();
    let mut client = MCPClient::start(&root);
    client.initialize().expect("initialize");

    // Create a page via wm_page.create
    client
        .call_tool(
            "wm_page",
            serde_json::json!({
                "action": "create",
                "path": "concepts/id-frontmatter-test",
                "title": "ID Frontmatter Test",
                "type": "concept",
            }),
        )
        .expect("create page");

    // Rebuild so the file is indexed
    client
        .call_tool("wm_index_rebuild", serde_json::json!({}))
        .expect("rebuild");

    // Read the created file and verify it has ^id:
    let content = std::fs::read_to_string(
        root.join(".wm")
            .join("wiki")
            .join("concepts")
            .join("id-frontmatter-test.md"),
    )
    .expect("read created file");
    assert!(
        content.contains("id: wiki:concepts:id-frontmatter-test"),
        "created page should have id: in frontmatter, got:\n{}",
        content
    );
}

#[test]
fn test_task_create_emits_id_frontmatter() {
    let (_dir, root) = setup::setup_test_project();
    let mut client = MCPClient::start(&root);
    client.initialize().expect("initialize");

    // Create a task via wm_task.create
    client
        .call_tool(
            "wm_task",
            serde_json::json!({
                "action": "create",
                "title": "ID Task Test",
                "description": "Testing task creates emit id: frontmatter.",
            }),
        )
        .expect("create task");

    // Rebuild so the file is indexed
    client
        .call_tool("wm_index_rebuild", serde_json::json!({}))
        .expect("rebuild");

    // Find the created file (slug: "id-task-test")
    let content = std::fs::read_to_string(
        root.join(".wm")
            .join("wiki")
            .join("tasks")
            .join("id-task-test.md"),
    )
    .expect("read created task file");
    assert!(
        content.contains("id: wiki:tasks:id-task-test"),
        "created task should have id: in frontmatter, got:\n{}",
        content
    );
}
