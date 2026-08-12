//! In-process MCP stdio server.
//!
//! `wm mcp` hosts the full tool registry in-process and serves it over rmcp
//! stdio — no daemon, no tokens, no readiness races. The registry is populated
//! by `wm_core::mcp::tools::register_all_tools`, the same single source of
//! truth used by wm-server, so `tools/list` is identical to the daemon's.

use std::sync::Arc;

use rmcp::{
    handler::server::ServerHandler,
    model::{
        CallToolRequestParams, CallToolResult, ContentBlock, ErrorCode, ErrorData,
        ListToolsResult, ServerCapabilities, ServerInfo, Tool,
    },
    service::RequestContext,
    transport::io::stdio,
    RoleServer, ServiceExt,
};

use wm_core::ToolRegistry;

struct McpServer {
    registry: Arc<ToolRegistry>,
}

impl ServerHandler for McpServer {
    fn get_info(&self) -> ServerInfo {
        let mut info = ServerInfo::default();
        info.capabilities = ServerCapabilities::builder().enable_tools().build();
        info.server_info = rmcp::model::Implementation::new("wm-engine", env!("CARGO_PKG_VERSION"));
        info.instructions = Some(
            "Call wm_initial at the start of every session; it injects project context.".into(),
        );
        info
    }

    async fn list_tools(
        &self,
        _request: Option<rmcp::model::PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, ErrorData> {
        let tools = self
            .registry
            .list_tools()
            .into_iter()
            .map(|value| {
                serde_json::from_value::<Tool>(value).map_err(|e| {
                    ErrorData::new(
                        ErrorCode::INTERNAL_ERROR,
                        format!("invalid tool definition: {e}"),
                        None,
                    )
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(ListToolsResult::with_all_items(tools))
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, ErrorData> {
        let arguments = request.arguments.unwrap_or_default();
        let result = self
            .registry
            .dispatch_async(&request.name, serde_json::Value::Object(arguments))
            .await;
        match result {
            Ok(data) => Ok(CallToolResult::success(vec![ContentBlock::text(
                data.to_string(),
            )])),
            Err(e) => {
                let text = serde_json::json!({ "error": e.message, "code": e.code }).to_string();
                Ok(CallToolResult::error(vec![ContentBlock::text(text)]))
            }
        }
    }
}

/// Serve the tool registry over rmcp stdio until the client disconnects.
pub async fn serve(registry: Arc<ToolRegistry>) -> Result<(), anyhow::Error> {
    let server = McpServer { registry };
    let service = server
        .serve(stdio())
        .await
        .map_err(|e| anyhow::anyhow!("failed to start rmcp server: {e}"))?;
    service
        .waiting()
        .await
        .map_err(|e| anyhow::anyhow!("rmcp server error: {e}"))?;
    Ok(())
}
