use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use rmcp::{
    model::{
        CallToolRequestParams, CallToolResult, ContentBlock, ErrorData,
        ListToolsResult, ServerInfo, Tool,
    },
    handler::server::ServerHandler,
    service::RequestContext,
    RoleServer,
    ServiceExt,
    transport::io::stdio,
};
use tracing::{error, info};

use crate::error::ToolError;

// ─── Handler type aliases ──────────────────────────────────────

/// A registered tool handler: sync closure that takes JSON params, returns JSON result.
pub type ToolHandler = Arc<dyn Fn(Value) -> Result<Value, ToolError> + Send + Sync>;

/// Audit callback type: (tool_name, action, result, duration_ms, error_message, entity_refs)
pub type AuditCallback =
    Arc<dyn Fn(&str, &str, &str, i64, Option<String>, Vec<String>) + Send + Sync>;

/// Permission check callback: (tool_name) -> allowed
pub type PermissionCheck = Arc<dyn Fn(&str) -> bool + Send + Sync>;

// ─── ToolRegistry ──────────────────────────────────────────────

/// Registry of all tool handlers — unchanged from the hand-rolled version.
pub struct ToolRegistry {
    handlers: Vec<(String, ToolHandler)>,
    /// Tool descriptions for list_tools response
    descriptions: HashMap<String, String>,
    /// Input JSON schemas per tool (for AI agent parameter discovery)
    schemas: HashMap<String, Value>,
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
            descriptions: HashMap::new(),
            schemas: HashMap::new(),
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

    /// Register a tool with description + input JSON schema (for AI agent discovery).
    pub fn register_with_schema(
        &mut self,
        name: &str,
        description: &str,
        schema: Value,
        handler: ToolHandler,
    ) {
        self.schemas.insert(name.to_string(), schema);
        self.descriptions
            .insert(name.to_string(), description.to_string());
        self.handlers.push((name.to_string(), handler));
    }

    /// Build the MCP-style tool list (used by the rmcp handler).
    pub fn list_tools(&self) -> Vec<Tool> {
        self.handlers
            .iter()
            .map(|(name, _)| {
                let desc = self.descriptions.get(name).cloned().unwrap_or_default();
                let schema = self
                    .schemas
                    .get(name)
                    .cloned()
                    .unwrap_or_else(|| serde_json::json!({"type": "object", "properties": {}}));
                let input_schema = schema.as_object().cloned().unwrap_or_default();
                Tool::new(name.clone(), desc, input_schema)
            })
            .collect()
    }

    /// Dispatch a tool call to the matching handler.
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

// ─── rmcp ServerHandler impl ───────────────────────────────────

impl ServerHandler for ToolRegistry {
    /// Server metadata sent during the initialize handshake.
    fn get_info(&self) -> ServerInfo {
        let mut info = ServerInfo::default();
        info.server_info = rmcp::model::Implementation::new(
            "wm-engine",
            env!("CARGO_PKG_VERSION"),
        );
        info.instructions = Some("Call wm_initial at the start of every session.".into());
        info
    }

    /// Handle tools/list — return all registered tools.
    async fn list_tools(
        &self,
        _request: Option<rmcp::model::PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, ErrorData> {
        Ok(ListToolsResult::with_all_items(self.list_tools()))
    }

    /// Handle tools/call — dispatch to the registered handler.
    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, ErrorData> {
        let name = &request.name;
        let args = request.arguments.unwrap_or_default();
        let args_value = Value::Object(args);

        // Catch panics in tool handlers (closure-based handlers may panic)
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            self.dispatch(name, args_value)
        }));

        match result {
            Ok(Ok(res)) => {
                let text = serde_json::to_string(&res).unwrap_or_default();
                Ok(CallToolResult::success(vec![ContentBlock::text(text)]))
            }
            Ok(Err(err)) => {
                // Tool-level error — caller-visible
                Ok(CallToolResult::error(vec![ContentBlock::text(
                    err.message,
                )]))
            }
            Err(panic_info) => {
                let msg = if let Some(s) = panic_info.downcast_ref::<&str>() {
                    s.to_string()
                } else if let Some(s) = panic_info.downcast_ref::<String>() {
                    s.clone()
                } else {
                    "unknown panic".to_string()
                };
                error!("Panic in tool handler '{}': {}", name, msg);
                Ok(CallToolResult::error(vec![ContentBlock::text(
                    "Internal server error",
                )]))
            }
        }
    }
}

// ─── Server entry point ────────────────────────────────────────

/// Serve the ToolRegistry via rmcp over stdio.
///
/// This is a blocking async call that runs until the client closes stdin
/// or the server encounters an error.
pub async fn serve_rmcp(registry: ToolRegistry) -> Result<(), anyhow::Error> {
    info!("Starting MCP server (rmcp stdio transport)");

    let service = registry
        .serve(stdio())
        .await
        .map_err(|e| anyhow::anyhow!("failed to start rmcp server: {e}"))?;

    info!("MCP server ready");

    // Wait for the client to disconnect or an error to occur.
    // Dropping the RunningService on Ctrl+C/shutdown is handled by the
    // caller via tokio::select! or Drop.
    service
        .waiting()
        .await
        .map_err(|e| anyhow::anyhow!("rmcp server error: {e}"))?;

    info!("MCP server stopped");
    Ok(())
}
