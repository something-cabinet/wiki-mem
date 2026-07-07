use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::io::BufRead;
use std::sync::Arc;
use tracing::{error, info};

use crate::error::ToolError;

/// JSON-RPC 2.0 request
#[derive(Debug, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub method: String,
    #[serde(default)]
    pub params: Option<Value>,
    pub id: Option<Value>,
}

/// JSON-RPC 2.0 response
#[derive(Debug, Serialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<Value>,
    pub id: Option<Value>,
}

impl JsonRpcResponse {
    pub fn success(id: Option<Value>, result: Value) -> Self {
        Self {
            jsonrpc: "2.0".into(),
            result: Some(result),
            error: None,
            id,
        }
    }

    pub fn error(id: Option<Value>, err: &ToolError) -> Self {
        Self {
            jsonrpc: "2.0".into(),
            result: None,
            error: Some(err.to_json()),
            id,
        }
    }

    pub fn parse_error(id: Option<Value>, msg: &str) -> Self {
        Self {
            jsonrpc: "2.0".into(),
            result: None,
            error: Some(serde_json::json!({ "code": -32700, "message": msg })),
            id,
        }
    }

    pub fn method_not_found(id: Option<Value>, method: &str) -> Self {
        Self {
            jsonrpc: "2.0".into(),
            result: None,
            error: Some(
                serde_json::json!({ "code": -32601, "message": format!("Method not found: {}", method) }),
            ),
            id,
        }
    }
}

/// Initialize response sent during MCP handshake
pub fn make_initialize_response() -> Value {
    serde_json::json!({
        "protocolVersion": "2024-11-05",
        "serverInfo": {
            "name": "wm-engine",
            "version": env!("CARGO_PKG_VERSION")
        },
        "capabilities": {
            "tools": {}
        },
        "instructions": "Call wm_initial at the start of every session."
    })
}

/// A registered tool handler
pub type ToolHandler = Arc<dyn Fn(Value) -> Result<Value, ToolError> + Send + Sync>;

/// Audit callback type: (tool_name, action, result, duration_ms, error_message, entity_refs)
pub type AuditCallback =
    Arc<dyn Fn(&str, &str, &str, i64, Option<String>, Vec<String>) + Send + Sync>;
/// Permission check callback: (tool_name) -> allowed
pub type PermissionCheck = Arc<dyn Fn(&str) -> bool + Send + Sync>;

/// Registry of all tool handlers
pub struct ToolRegistry {
    handlers: Vec<(String, ToolHandler)>,
    /// Tool descriptions for list_tools response
    descriptions: std::collections::HashMap<String, String>,
    audit: Option<AuditCallback>,
    /// Optional permission check: returns true if the action is permitted.
    /// Called with the tool name (e.g. "wm_page.delete") before execution.
    pub check_permission: Option<PermissionCheck>,
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self {
            handlers: Vec::new(),
            descriptions: std::collections::HashMap::new(),
            audit: None,
            check_permission: None,
        }
    }

    pub fn set_audit(&mut self, cb: AuditCallback) {
        self.audit = Some(cb);
    }

    pub fn set_permission_check(&mut self, cb: Arc<dyn Fn(&str) -> bool + Send + Sync>) {
        self.check_permission = Some(cb);
    }

    pub fn register(&mut self, name: &str, handler: ToolHandler) {
        self.handlers.push((name.to_string(), handler));
    }

    /// Register a tool with a human-readable description
    pub fn register_with_desc(
        &mut self,
        name: &str,
        description: &str,
        handler: ToolHandler,
    ) {
        self.descriptions
            .insert(name.to_string(), description.to_string());
        self.handlers.push((name.to_string(), handler));
    }

    pub fn list_tools(&self) -> Value {
        let tools: Vec<Value> = self
            .handlers
            .iter()
            .map(|(name, _)| {
                let desc = self.descriptions.get(name).map(|s| s.as_str()).unwrap_or("");
                serde_json::json!({
                    "name": name,
                    "description": desc,
                    "inputSchema": {
                        "type": "object",
                        "properties": {}
                    }
                })
            })
            .collect();
        serde_json::json!({ "tools": tools })
    }

    pub fn dispatch(&self, method: &str, params: Value) -> Result<Value, ToolError> {
        // Permission check before execution
        if let Some(ref check) = self.check_permission {
            if !check(method) {
                return Err(ToolError::internal("Action not permitted"));
            }
        }

        for (name, handler) in &self.handlers {
            if name == method {
                let start = std::time::Instant::now();
                let result = handler(params);
                let duration_ms = start.elapsed().as_millis() as i64;
                // Emit audit event for non-system tools
                if let Some(ref audit) = self.audit {
                    if *name != "wm_help" && *name != "wm_initial" {
                        let error_msg = match &result {
                            Ok(_) => None,
                            Err(e) => Some(e.to_string()),
                        };
                        let status = if error_msg.is_some() { "error" } else { "ok" };
                        let action = name.split('.').nth(1).unwrap_or("unknown");
                        audit(name, action, status, duration_ms, error_msg, Vec::new());
                    }
                }
                return result;
            }
        }
        Err(ToolError::invalid_action(&[method]))
    }
}

/// Run the stdio MCP transport loop
pub async fn run_transport(registry: Arc<ToolRegistry>) -> Result<(), anyhow::Error> {
    let stdin = std::io::stdin();
    let reader = std::io::BufReader::new(stdin.lock());

    for line in reader.lines() {
        let line = match line {
            Ok(l) => l,
            Err(e) => {
                error!("stdin read error: {}", e);
                continue;
            }
        };

        if line.trim().is_empty() {
            continue;
        }

        // Parse JSON-RPC request
        let request: JsonRpcRequest = match serde_json::from_str(&line) {
            Ok(r) => r,
            Err(e) => {
                let resp = JsonRpcResponse::parse_error(None, &format!("Parse error: {}", e));
                let out = serde_json::to_string(&resp).unwrap_or_default();
                println!("{}", out);
                continue;
            }
        };

        let response = match request.method.as_str() {
            "initialize" => JsonRpcResponse::success(request.id, make_initialize_response()),
            "tools/list" => JsonRpcResponse::success(request.id, registry.list_tools()),
            "tools/call" => {
                let params = request.params.unwrap_or(serde_json::json!({}));
                let tool_name = params.get("name").and_then(|v| v.as_str()).unwrap_or("");
                let tool_args = params
                    .get("arguments")
                    .cloned()
                    .unwrap_or(serde_json::json!({}));

                // Catch panics in tool handlers
                let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    registry.dispatch(tool_name, tool_args)
                }));

                match result {
                    Ok(Ok(res)) => JsonRpcResponse::success(request.id, res),
                    Ok(Err(err)) => JsonRpcResponse::error(request.id, &err),
                    Err(panic_info) => {
                        let msg = if let Some(s) = panic_info.downcast_ref::<&str>() {
                            s.to_string()
                        } else if let Some(s) = panic_info.downcast_ref::<String>() {
                            s.clone()
                        } else {
                            "unknown panic".to_string()
                        };
                        error!("Panic in tool handler: {}", msg);
                        JsonRpcResponse::error(
                            request.id,
                            &ToolError::internal("Internal server error"),
                        )
                    }
                }
            }
            "notifications/initialized" => {
                info!("Client initialized");
                continue;
            }
            _ => JsonRpcResponse::method_not_found(request.id, &request.method),
        };

        let json = serde_json::to_string(&response).unwrap_or_default();
        println!("{}", json);
    }

    Ok(())
}
