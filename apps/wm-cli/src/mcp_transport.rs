use rmcp::{
    handler::server::ServerHandler,
    model::{
        CallToolRequestParams, CallToolResult, ContentBlock, ErrorCode, ErrorData, ListToolsResult,
        ServerCapabilities, ServerInfo,
    },
    service::RequestContext,
    transport::io::stdio,
    RoleServer, ServiceExt,
};
use serde_json::Value;
use tracing::info;

use wm_core::ToolRegistry;

pub struct McpServer(pub ToolRegistry);

impl ServerHandler for McpServer {
    fn get_info(&self) -> ServerInfo {
        let mut info = ServerInfo::default();
        info.capabilities = ServerCapabilities::builder().enable_tools().build();
        info.server_info = rmcp::model::Implementation::new("wm-engine", env!("CARGO_PKG_VERSION"));
        info.instructions = Some("Call wm_initial at the start of every session.".into());
        info
    }

    async fn list_tools(
        &self,
        _request: Option<rmcp::model::PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, ErrorData> {
        Ok(ListToolsResult::with_all_items(self.0.list_tools()))
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, ErrorData> {
        let name = &request.name;
        let args = request.arguments.unwrap_or_default();
        let args_value = Value::Object(args);

        if !self.0.has_tool(name) {
            return Err(ErrorData::new(
                ErrorCode::METHOD_NOT_FOUND,
                format!("Unknown tool: {name}"),
                None,
            ));
        }

        match self.0.dispatch_async(name, args_value).await {
            Ok(res) => {
                let text = serde_json::to_string(&res).unwrap_or_default();
                Ok(CallToolResult::success(vec![ContentBlock::text(text)]))
            }
            Err(err) => {
                let text = serde_json::to_string(&err.to_json()).unwrap_or_default();
                Ok(CallToolResult::error(vec![ContentBlock::text(text)]))
            }
        }
    }
}

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
