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

use wm_core::engine::{EngineState, PageStatus, PageType};
use wm_core::ToolRegistry;

fn runtime_context_block(engine: &EngineState) -> ContentBlock {
    let version = env!("CARGO_PKG_VERSION");
    let snapshot = engine.graph.load();
    let graph = &snapshot.0;

    let core_titles: Vec<&str> = graph
        .node_indices()
        .filter(|i| graph[*i].page_type == PageType::Core)
        .map(|i| graph[i].title.as_str())
        .take(5)
        .collect();
    let core_count = graph.node_indices().filter(|i| graph[*i].page_type == PageType::Core).count();
    let active_task_count = graph
        .node_indices()
        .filter(|i| {
            graph[*i].page_type == PageType::Task
                && (graph[*i].status == PageStatus::Todo
                    || graph[*i].status == PageStatus::InProgress
                    || graph[*i].status == PageStatus::Blocked)
        })
        .count();
    // snapshot guard drops at scope end

    let core_line = if core_titles.is_empty() {
        format!("Core pages: {}", core_count)
    } else {
        format!("Core pages: {} ({})", core_titles.join(", "), core_count)
    };

    let text = format!(
        "[Wiki Memory Engine v{}]\n{} | Active tasks: {}\n",
        version, core_line, active_task_count
    );
    ContentBlock::text(text)
}

pub struct McpServer {
    registry: ToolRegistry,
    engine: Arc<EngineState>,
    first_call_served: AtomicBool,
}

impl ServerHandler for McpServer {
    fn get_info(&self) -> ServerInfo {
        let mut info = ServerInfo::default();
        info.capabilities = ServerCapabilities::builder().enable_tools().build();
        info.server_info = rmcp::model::Implementation::new("wm-engine", env!("CARGO_PKG_VERSION"));
        info.instructions = Some(
            "Call wm_initial at the start of every session. First tool call injects project context.".into(),
        );
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

                if !self.first_call_served.swap(true, Ordering::Relaxed) {
                    content.push(runtime_context_block(&self.engine));
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
