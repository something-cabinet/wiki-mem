//! MCP stdio → HTTP proxy to the wm-server daemon.
//!
//! `wm mcp` no longer hosts tools in-process. This module runs a thin rmcp
//! stdio server that forwards `list_tools` / `call_tool` to the privileged
//! `/api/mcp/*` channel of a co-located `wm-server` daemon (task #41 / wiki
//! task 22ed6a).
//!
//! Design notes:
//! - **Dynamic discovery**: there is no `STATIC_TOOLS` array; `list_tools`
//!   fetches the daemon's registry via `/api/mcp/tools/list` and caches it.
//! - **Parallel spawn**: daemon detection/spawning runs in the background,
//!   concurrently with the rmcp handshake. The first daemon-dependent call
//!   awaits a shared readiness cell (bounded ~10s).
//! - **No blocking on the runtime**: all ureq HTTP calls run inside
//!   `tokio::task::spawn_blocking`.
//! - **Error mapping**: `{success:false,...}` → `CallToolResult::error`
//!   (`is_error: true`); daemon-unreachable → rmcp `ErrorData`.
//! - **Token lifecycle**: the MCP token is read from `.wm/state/mcp-token`
//!   after the health check passes; on any 401 the token file is re-read and
//!   the request is retried once (the daemon rotates tokens on restart).

use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use rmcp::{
    handler::server::ServerHandler,
    model::{
        CallToolRequestParams, CallToolResult, ContentBlock, ErrorCode, ErrorData, ListToolsResult,
        ServerCapabilities, ServerInfo, Tool,
    },
    service::RequestContext,
    transport::io::stdio,
    RoleServer, ServiceExt,
};
use serde_json::{json, Value};
use tokio::sync::Notify;
use tracing::{info, warn};

use crate::http_client::{self, DaemonClient, MCP_TOOLS_CALL_PATH, MCP_TOOLS_LIST_PATH};

/// Shared state for the proxy handler: readiness cell for the daemon plus the
/// dynamic tool cache.
struct ProxyState {
    project_root: PathBuf,
    daemon: Mutex<Option<Arc<DaemonClient>>>,
    error: Mutex<Option<String>>,
    notify: Notify,
    tools_cache: Mutex<Option<Vec<Tool>>>,
}

impl ProxyState {
    fn new(project_root: PathBuf) -> Self {
        Self {
            project_root,
            daemon: Mutex::new(None),
            error: Mutex::new(None),
            notify: Notify::new(),
            tools_cache: Mutex::new(None),
        }
    }

    /// Kick off daemon detection/spawning in the background so it runs in
    /// parallel with the rmcp handshake.
    fn spawn_daemon_detection(self: &Arc<Self>) {
        let state = self.clone();
        tokio::spawn(async move {
            let root = state.project_root.clone();
            let outcome = tokio::task::spawn_blocking(move || http_client::detect_server_url(&root))
                .await
                .map_err(|e| anyhow::anyhow!("daemon detection task panicked: {e}"))
                .and_then(|result| result);
            match outcome {
                Ok(info) => *state.daemon.lock().unwrap() = Some(Arc::new(info)),
                Err(e) => {
                    warn!("wm-server unavailable: {e:#}");
                    *state.error.lock().unwrap() = Some(e.to_string());
                }
            }
            state.notify.notify_waiters();
        });
    }

    /// Resolve the daemon, waiting (bounded) until detection completes.
    async fn await_daemon(&self) -> Result<Arc<DaemonClient>, ErrorData> {
        let deadline = Instant::now()
            + Duration::from_secs(http_client::DAEMON_READY_TIMEOUT_SECS);
        loop {
            if let Some(info) = self.daemon.lock().unwrap().clone() {
                return Ok(info);
            }
            if let Some(e) = self.error.lock().unwrap().clone() {
                return Err(ErrorData::new(
                    ErrorCode::INTERNAL_ERROR,
                    format!("wm-server unavailable: {e}"),
                    None,
                ));
            }
            let now = Instant::now();
            if now >= deadline {
                return Err(self.timeout_error());
            }
            let notified = self.notify.notified();
            tokio::pin!(notified);
            if tokio::time::timeout(deadline.saturating_duration_since(now), &mut notified)
                .await
                .is_err()
            {
                return Err(self.timeout_error());
            }
        }
    }

    fn timeout_error(&self) -> ErrorData {
        ErrorData::new(
            ErrorCode::INTERNAL_ERROR,
            format!(
                "timed out after {}s waiting for wm-server",
                http_client::DAEMON_READY_TIMEOUT_SECS
            ),
            None,
        )
    }

    fn mcp_token_path(&self) -> PathBuf {
        http_client::mcp_token_path(&self.project_root)
    }
}

struct ProxyServer {
    state: Arc<ProxyState>,
}

impl ServerHandler for ProxyServer {
    fn get_info(&self) -> ServerInfo {
        let mut info = ServerInfo::default();
        info.capabilities = ServerCapabilities::builder().enable_tools().build();
        info.server_info = rmcp::model::Implementation::new("wm-engine", env!("CARGO_PKG_VERSION"));
        info.instructions = Some(
            "Call wm_initial at the start of every session; the daemon injects project context.".into(),
        );
        info
    }

    async fn list_tools(
        &self,
        _request: Option<rmcp::model::PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, ErrorData> {
        if let Some(tools) = self.state.tools_cache.lock().unwrap().clone() {
            return Ok(ListToolsResult::with_all_items(tools));
        }

        let info = self.state.await_daemon().await?;
        let base = info.base_url.clone();
        let token_cell = info.token.clone();
        let token_path = self.state.mcp_token_path();
        let payload = json!({});

        let response = tokio::task::spawn_blocking(move || {
            http_client::post_json_with_retry(
                &base,
                MCP_TOOLS_LIST_PATH,
                &payload,
                &token_cell,
                &token_path,
            )
        })
        .await
        .map_err(|e| {
            ErrorData::new(
                ErrorCode::INTERNAL_ERROR,
                format!("list_tools task failed: {e}"),
                None,
            )
        })?;

        let (status, text) = response.map_err(|e| {
            ErrorData::new(
                ErrorCode::INTERNAL_ERROR,
                format!("wm-server unreachable: {e}"),
                None,
            )
        })?;
        if !(200..300).contains(&status) {
            return Err(ErrorData::new(
                ErrorCode::INTERNAL_ERROR,
                format!("wm-server rejected tools/list with HTTP {status}: {text}"),
                None,
            ));
        }

        let parsed: Value = serde_json::from_str(&text).map_err(|e| {
            ErrorData::new(
                ErrorCode::INTERNAL_ERROR,
                format!("invalid tools/list response: {e}"),
                None,
            )
        })?;
        if parsed.get("success").and_then(Value::as_bool) != Some(true) {
            let msg = parsed["error"]
                .as_str()
                .unwrap_or("tools/list failed on daemon")
                .to_string();
            return Err(ErrorData::new(ErrorCode::INTERNAL_ERROR, msg, None));
        }

        let tools: Vec<Tool> = serde_json::from_value(parsed["data"]["tools"].clone()).map_err(
            |e| {
                ErrorData::new(
                    ErrorCode::INTERNAL_ERROR,
                    format!("invalid tools payload from daemon: {e}"),
                    None,
                )
            },
        )?;
        self.state
            .tools_cache
            .lock()
            .unwrap()
            .clone_from(&Some(tools.clone()));
        Ok(ListToolsResult::with_all_items(tools))
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, ErrorData> {
        let name = &request.name;
        let arguments = request.arguments.unwrap_or_default();
        let payload = json!({ "name": name, "arguments": Value::Object(arguments) });

        let info = self.state.await_daemon().await?;
        let base = info.base_url.clone();
        let token_cell = info.token.clone();
        let token_path = self.state.mcp_token_path();

        let response = tokio::task::spawn_blocking(move || {
            http_client::post_json_with_retry(
                &base,
                MCP_TOOLS_CALL_PATH,
                &payload,
                &token_cell,
                &token_path,
            )
        })
        .await
        .map_err(|e| {
            ErrorData::new(
                ErrorCode::INTERNAL_ERROR,
                format!("call_tool task failed: {e}"),
                None,
            )
        })?;

        let (status, text) = response.map_err(|e| {
            ErrorData::new(
                ErrorCode::INTERNAL_ERROR,
                format!("wm-server unreachable: {e}"),
                None,
            )
        })?;
        if !(200..300).contains(&status) {
            return Err(ErrorData::new(
                ErrorCode::INTERNAL_ERROR,
                format!("wm-server rejected tools/call with HTTP {status}: {text}"),
                None,
            ));
        }

        let parsed: Value = serde_json::from_str(&text).map_err(|e| {
            ErrorData::new(
                ErrorCode::INTERNAL_ERROR,
                format!("invalid tools/call response: {e}"),
                None,
            )
        })?;

        if parsed.get("success").and_then(Value::as_bool) == Some(true) {
            let text = parsed["data"].to_string();
            Ok(CallToolResult::success(vec![ContentBlock::text(text)]))
        } else {
            let error = parsed["error"].as_str().unwrap_or("tool error");
            let code = parsed["code"].as_str().unwrap_or("TOOL_ERROR");
            let text = json!({ "error": error, "code": code }).to_string();
            Ok(CallToolResult::error(vec![ContentBlock::text(text)]))
        }
    }
}

/// Serve the MCP proxy over stdio for the given project root.
pub async fn serve_proxy(project_root: PathBuf) -> Result<(), anyhow::Error> {
    info!("Starting MCP proxy (rmcp stdio transport → wm-server daemon)");

    let state = Arc::new(ProxyState::new(project_root));
    state.spawn_daemon_detection();

    let server = ProxyServer { state };
    let service = server
        .serve(stdio())
        .await
        .map_err(|e| anyhow::anyhow!("failed to start rmcp server: {e}"))?;

    info!("MCP proxy ready");

    service
        .waiting()
        .await
        .map_err(|e| anyhow::anyhow!("rmcp server error: {e}"))?;

    info!("MCP proxy stopped");
    Ok(())
}
