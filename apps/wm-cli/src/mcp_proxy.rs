// ─── MCP Proxy Handlers ────────────────────────────────
//
// Registers proxy handlers for all known MCP tools that
// route to the wm-server HTTP API instead of using direct
// in-process handlers.

use std::path::PathBuf;
use std::sync::Arc;
use wm_core::error::ToolError;
use wm_core::ToolRegistry;

/// Static list of all known MCP tools
const STATIC_TOOLS: &[&str] = &[
    "wm_initial",
    "wm_search.query",
    "wm_search.retrieve",
    "wm_search.resolve",
    "wm_page.get",
    "wm_page.list",
    "wm_page.create",
    "wm_page.update",
    "wm_page.delete",
    "wm_page.link",
    "wm_page.unlink",
    "wm_task.board",
    "wm_task.list",
    "wm_task.create",
    "wm_task.update",
    "wm_task.delete",
    "wm_graph.stats",
    "wm_graph.neighbors",
    "wm_graph.path",
    "wm_graph.subgraph",
    "wm_graph.full",
    "wm_memory.list",
    "wm_memory.get",
    "wm_memory.add",
    "wm_index.rebuild",
    "wm_index.status",
    "wm_index.embed",
    "wm_template.list",
    "wm_template.get",
    "wm_template.create",
    "wm_time.start",
    "wm_time.stop",
    "wm_time.report",
    "wm_source.add",
    "wm_source.list",
    "wm_source.process",
    "wm_source.complete",
    "wm_lint.check",
    "wm_validate.check",
    "wm_help",
    "wm_version",
    "wm_lsp.status",
    "wm_lsp.definition",
    "wm_lsp.references",
    "wm_lsp.hover",
    "wm_lsp.implementations",
    "wm_lsp.workspace_symbols",
    "wm_lsp.diagnostics",
    "wm_lsp.rename",
];

/// Register proxy handlers for all tools, routing to wm-server HTTP API
pub fn register_proxy_handlers(registry: &mut ToolRegistry, server_url: &str) {
    for tool_name in STATIC_TOOLS {
        let url = format!("{}/api/tools/{}", server_url, tool_name);
        let url2 = url.clone(); // clone for the closure
        registry.register(
            tool_name,
            Arc::new(move |params: serde_json::Value| -> Result<serde_json::Value, ToolError> {
                let response = ureq::post(&url2)
                    .set("Content-Type", "application/json")
                    .send_json(&params)
                    .map_err(|e| ToolError::internal(&format!("HTTP request failed: {e}")))?;

                let body: serde_json::Value = response
                    .into_json()
                    .map_err(|e| ToolError::internal(&format!("JSON parse failed: {e}")))?;

                Ok(body)
            }),
        );
    }
}

/// Detect the wm-server URL from .wm/server.json
pub fn detect_server_url(project_root: &std::path::Path) -> Result<String, ToolError> {
    let config_path = project_root.join(".wm/server.json");

    // Try to find running server
    if let Ok(content) = std::fs::read_to_string(&config_path) {
        if let Ok(config) = serde_json::from_str::<serde_json::Value>(&content) {
            if let Some(port) = config.get("port").and_then(|v| v.as_u64()) {
                let url = format!("http://localhost:{}", port);
                if let Ok(resp) = ureq::get(&format!("{}/api/health", &url)).call() {
                    if resp.status() == 200 {
                        return Ok(url);
                    }
                }
            }
        }
    }

    // Not running — spawn it
    let port = 4090u16;
    ensure_server(project_root, port)?;
    Ok(format!("http://localhost:{}", port))
}

/// Spawn wm-server if not already running, wait for health check
fn ensure_server(_project_root: &std::path::Path, port: u16) -> Result<(), ToolError> {
    let url = format!("http://localhost:{}/api/health", port);

    // One final health check before spawning
    if let Ok(resp) = ureq::get(&url).call() {
        if resp.status() == 200 {
            return Ok(()); // already running
        }
    }

    // Find wm-server binary
    let server_binary = std::env::current_exe()
        .unwrap_or_default()
        .parent()
        .map(|p| {
            let mut path = p.to_path_buf();
            path.push(if cfg!(windows) { "wm-server.exe" } else { "wm-server" });
            path
        })
        .unwrap_or_else(|| {
            // Fallback: look in PATH or default location
            PathBuf::from(if cfg!(windows) { "wm-server.exe" } else { "wm-server" })
        });

    if !server_binary.exists() {
        return Err(ToolError::not_found("wm-server binary",
            &format!("Not found at {}. Build with: cargo build -p wm-server", server_binary.display())));
    }

    let _child = std::process::Command::new(&server_binary)
        .arg("--port")
        .arg(port.to_string())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .map_err(|e| ToolError::internal(&format!("Failed to spawn wm-server: {e}")))?;

    // Wait for health check (up to 6 seconds)
    for _ in 0..30 {
        std::thread::sleep(std::time::Duration::from_millis(200));
        if let Ok(resp) = ureq::get(&url).call() {
            if resp.status() == 200 {
                // Server is running — child process handle dropped, process continues
                return Ok(());
            }
        }
    }

    Err(ToolError::internal("wm-server failed to start within 6 seconds"))
}
