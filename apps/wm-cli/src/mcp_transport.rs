// ─── MCP Transport Layer ────────────────────────────────
//
// rmcp ServerHandler implementation + stdio transport for ToolRegistry.
// Uses a newtype wrapper (McpServer) to satisfy Rust orphan rules,
// since impl foreign trait + foreign type can't live in a third crate.
//
// Lives in wm-cli so wm-server doesn't drag in rmcp server deps.

use rmcp::{
    handler::server::ServerHandler,
    model::{
        CallToolRequestParams, CallToolResult, ContentBlock, ErrorCode, ErrorData,
        ListToolsResult, ServerCapabilities, ServerInfo,
    },
    service::RequestContext,
    RoleServer,
    ServiceExt,
    transport::io::stdio,
};
use serde_json::Value;
use tracing::info;

use wm_core::ToolRegistry;

// ─── Newtype wrapper for orphan-rule compliance ───────────────

/// Wraps ToolRegistry so we can implement rmcp's ServerHandler for it
/// from a third crate (wm-cli) without violating Rust orphan rules.
pub struct McpServer(pub ToolRegistry);

// ─── rmcp ServerHandler impl ───────────────────────────────────

impl ServerHandler for McpServer {
    /// Server metadata sent during the initialize handshake.
    /// ⚠️ CRITICAL: capabilities must advertise tools support.
    fn get_info(&self) -> ServerInfo {
        let mut info = ServerInfo::default();
        info.capabilities = ServerCapabilities::builder()
            .enable_tools()
            .build();
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
        Ok(ListToolsResult::with_all_items(self.0.list_tools()))
    }

    /// Handle tools/call — dispatch to the registered handler (sync or async).
    ///
    /// Error handling follows MCP best practices:
    /// - **Dispatch-miss** (unknown tool name, not in registry) → JSON-RPC `Err(ErrorData)`
    /// - **Handler-returned `ToolError`** → `Ok(CallToolResult)` with `isError: true`
    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, ErrorData> {
        let name = &request.name;
        let args = request.arguments.unwrap_or_default();
        let args_value = Value::Object(args);

        // Check tool existence before dispatch so we can return a clean protocol error
        // for unknown tools rather than conflating dispatch-miss with handler errors.
        if !self.0.has_tool(name) {
            return Err(ErrorData::new(
                ErrorCode::METHOD_NOT_FOUND,
                format!("Unknown tool: {name}"),
                None,
            ));
        }

        // dispatch_async handles both async and sync (with block_in_place + panic catching)
        match self.0.dispatch_async(name, args_value).await {
            Ok(res) => {
                let text = serde_json::to_string(&res).unwrap_or_default();
                Ok(CallToolResult::success(vec![ContentBlock::text(text)]))
            }
            Err(err) => {
                // Handler-returned ToolError — use isError: true so the caller sees the message
                let text = serde_json::to_string(&err.to_json()).unwrap_or_default();
                Ok(CallToolResult::error(vec![ContentBlock::text(text)]))
            }
        }
    }
}

// ─── Server entry point ────────────────────────────────────────

/// Serve the ToolRegistry via rmcp over stdio.
/// This is a blocking async call that runs until the client closes stdin
/// or the server encounters an error.
pub async fn serve_rmcp(registry: ToolRegistry) -> Result<(), anyhow::Error> {
    info!("Starting MCP server (rmcp stdio transport)");

    let server = McpServer(registry);
    let service = server
        .serve(stdio())
        .await
        .map_err(|e| anyhow::anyhow!("failed to start rmcp server: {e}"))?;

    info!("MCP server ready");

    service
        .waiting()
        .await
        .map_err(|e| anyhow::anyhow!("rmcp server error: {e}"))?;

    info!("MCP server stopped");
    Ok(())
}
