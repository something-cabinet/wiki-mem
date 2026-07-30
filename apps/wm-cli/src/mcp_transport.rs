use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

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

use wm_core::engine::{EngineState, PageType};
use wm_core::ToolRegistry;

pub struct McpServer {
    pub registry: ToolRegistry,
    pub engine: Arc<EngineState>,
    pub first_call_served: AtomicBool,
}

impl ServerHandler for McpServer {
    fn get_info(&self) -> ServerInfo {
        let mut info = ServerInfo::default();
        info.capabilities = ServerCapabilities::builder().enable_tools().build();
        info.server_info = rmcp::model::Implementation::new("wm-engine", env!("CARGO_PKG_VERSION"));
        info.instructions = Some("Wiki Memory Engine MCP server. First tool call in a session automatically injects runtime context.".into());
        info
    }

    async fn list_tools(
        &self,
        _request: Option<rmcp::model::PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, ErrorData> {
        Ok(ListToolsResult::with_all_items(self.registry.list_tools()))
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, ErrorData> {
        let name = &request.name;
        let args = request.arguments.unwrap_or_default();
        let args_value = Value::Object(args);

        if !self.registry.has_tool(name) {
            return Err(ErrorData::new(
                ErrorCode::METHOD_NOT_FOUND,
                format!("Unknown tool: {name}"),
                None,
            ));
        }

        match self.registry.dispatch_async(name, args_value).await {
            Ok(res) => {
                let text = serde_json::to_string(&res).unwrap_or_default();
                let mut content = vec![ContentBlock::text(text)];

                // Inject runtime context on first tool call of the session
                if !self.first_call_served.swap(true, Ordering::SeqCst) {
                    let version = env!("CARGO_PKG_VERSION");
                    let snapshot = self.engine.graph.load();
                    let (ref graph, _) = &**snapshot;
                    let core_count = graph.node_indices().filter(|i| {
                        graph[*i].page_type == PageType::Core
                    }).count();
                    let task_count = graph.node_indices().filter(|i| {
                        graph[*i].page_type == PageType::Task
                    }).count();
                    drop(snapshot);
                    let injected = format!(
                        "[Wiki Memory Engine v{}]\nCore pages: {} | Tasks: {}\n",
                        version, core_count, task_count
                    );
                    content.insert(0, ContentBlock::text(injected));
                }

                Ok(CallToolResult::success(content))
            }
            Err(err) => {
                let text = serde_json::to_string(&err.to_json()).unwrap_or_default();
                Ok(CallToolResult::error(vec![ContentBlock::text(text)]))
            }
        }
    }
}

pub async fn serve_rmcp(registry: ToolRegistry, engine: Arc<EngineState>) -> Result<(), anyhow::Error> {
    info!("Starting MCP server (rmcp stdio transport)");

    let server = McpServer {
        registry,
        engine,
        first_call_served: AtomicBool::new(false),
    };
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
