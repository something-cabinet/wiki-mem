#[path = "helpers/mcp.rs"]
mod helpers;
use helpers::MCPClient;

#[path = "helpers/setup.rs"]
mod setup;

fn setup_mcp_test() -> (tempfile::TempDir, MCPClient) {
    let (dir, root) = setup::setup_test_project();
    let client = MCPClient::start(&root);
    (dir, client)
}

fn expect_field<'a>(value: &'a serde_json::Value, key: &str) -> &'a serde_json::Value {
    value
        .get(key)
        .unwrap_or_else(|| panic!("expected {} in response, got: {}", key, value))
}

#[test]
fn test_error_data_structured_on_not_found() {
    let (_dir, mut client) = setup_mcp_test();
    client.initialize().expect("initialize");

    let resp = client
        .send_request_raw(
            "tools/call",
            serde_json::json!({
                "name": "wm_page",
                "arguments": { "action": "get", "id": "nonexistent:id" },
            }),
        )
        .expect("tools/call failed");

    let result = expect_field(&resp, "result");
    assert_eq!(
        result.get("isError").and_then(|v| v.as_bool()),
        Some(true),
        "tool-level failure must surface as isError, got: {}",
        resp
    );
    let text = result
        .get("content")
        .and_then(|c| c.as_array())
        .and_then(|a| a.first())
        .and_then(|c| c.get("text"))
        .and_then(|t| t.as_str())
        .expect("error content text");
    let data: serde_json::Value = serde_json::from_str(text).expect("parse error content");
    assert_eq!(
        data.get("code").and_then(|v| v.as_str()),
        Some("NOT_FOUND")
    );
    let message = data.get("error").and_then(|v| v.as_str()).unwrap_or("");
    assert!(
        message.contains("not found"),
        "expected message to mention 'not found', got: {}",
        message
    );

    let err = client
        .call_tool(
            "wm_page",
            serde_json::json!({ "action": "get", "id": "nonexistent:id" }),
        )
        .unwrap_err();
    assert!(
        err.contains("not found"),
        "expected error message to mention 'not found', got: {}",
        err
    );
}

#[test]
fn test_error_data_structured_on_invalid_input() {
    let (_dir, mut client) = setup_mcp_test();
    client.initialize().expect("initialize");

    let tools = client.list_tools().expect("list_tools");
    assert!(
        tools.contains(&"wm_page".to_string()),
        "expected wm_page tool to be registered"
    );

    let resp = client
        .send_request_raw(
            "tools/call",
            serde_json::json!({
                "name": "wm_page",
                "arguments": { "action": "fly" },
            }),
        )
        .expect("tools/call failed");

    let result = expect_field(&resp, "result");
    assert_eq!(
        result.get("isError").and_then(|v| v.as_bool()),
        Some(true),
        "invalid input must surface as isError, got: {}",
        resp
    );
    let text = result
        .get("content")
        .and_then(|c| c.as_array())
        .and_then(|a| a.first())
        .and_then(|c| c.get("text"))
        .and_then(|t| t.as_str())
        .expect("error content text");
    let data: serde_json::Value = serde_json::from_str(text).expect("parse error content");
    assert_eq!(
        data.get("code").and_then(|v| v.as_str()),
        Some("SERDE_ERROR")
    );
    let message = data.get("error").and_then(|v| v.as_str()).unwrap_or("");
    assert!(
        message.contains("fly"),
        "expected message to mention the invalid action, got: {}",
        message
    );
}
