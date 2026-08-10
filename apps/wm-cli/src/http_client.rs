//! Shared HTTP client for talking to the wm-server daemon.
//!
//! Two consumers:
//! - `wm mcp` proxy (`mcp_proxy.rs`) — async stdio→HTTP forwarder.
//! - Migrated CLI commands (`main.rs`) — synchronous `call_tool` entry point.
//!
//! Both share the same discovery/spawn-if-absent logic: read `.wm/server.json`
//! for the port (fallback `127.0.0.1:4090`), health-check `/api/health`, and
//! spawn a co-located `wm-server` binary if nothing is live. The privileged
//! MCP channel (`POST /api/mcp/tools/call`) is used because it is the complete
//! surface (reads + writes) behind the MCP token.
//!
//! Token lifecycle: the MCP token is read from `.wm/state/mcp-token` after the
//! health check passes; on any 401 the token file is re-read and the request is
//! retried once (the daemon rotates tokens on restart).

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant};

use serde_json::{json, Value};
use tracing::info;

use wm_constants::*;

pub(crate) const MCP_TOOLS_LIST_PATH: &str = "/api/mcp/tools/list";
pub(crate) const MCP_TOOLS_CALL_PATH: &str = "/api/mcp/tools/call";
const TOKEN_HEADER: &str = "x-wm-token";
const MCP_TOKEN_FILE: &str = "mcp-token";
pub(crate) const DAEMON_READY_TIMEOUT_SECS: u64 = 10;
/// Per-request HTTP timeout. Generous because index rebuilds (and other heavy
/// tools) can legitimately take a while on the daemon side.
const HTTP_TIMEOUT: Duration = Duration::from_secs(300);
const PROBE_INTERVAL_MS: u64 = 100;

/// Resolved daemon endpoint plus the credential cell used for token rotation.
pub(crate) struct DaemonClient {
    pub(crate) base_url: String,
    pub(crate) token: Arc<Mutex<Option<String>>>,
    pub(crate) token_path: PathBuf,
    /// Keeps the spawned daemon referenced for the caller lifetime; dropped (not
    /// killed) on exit so the daemon persists for future clients.
    #[allow(dead_code, reason = "held to keep the spawned daemon alive")]
    pub(crate) child: Option<std::process::Child>,
}

/// Resolve the daemon for `root`: read `.wm/server.json`, health-check the
/// recorded port (fallback `127.0.0.1:4090`), and spawn a co-located
/// `wm-server` binary if nothing is live.
///
/// Blocking — call from `spawn_blocking` or a sync command handler.
pub(crate) fn detect_server_url(root: &Path) -> anyhow::Result<DaemonClient> {
    let port = recorded_port(root).unwrap_or(DEFAULT_PORT);
    let base_url = format!("http://{LOCALHOST_ADDR}:{port}");

    if crate::http_status(port, "/api/health").is_some_and(|c| (200..300).contains(&c)) {
        let token = read_token(root)?;
        return Ok(DaemonClient {
            base_url,
            token: Arc::new(Mutex::new(Some(token))),
            token_path: mcp_token_path(root),
            child: None,
        });
    }

    let binary = crate::resolve_server_binary();
    info!("Spawning wm-server ({}) on port {port}", binary.display());
    let mut child = std::process::Command::new(&binary)
        .arg("--port")
        .arg(port.to_string())
        .current_dir(root)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .map_err(|e| {
            anyhow::anyhow!(
                "failed to start wm-server ({}) on port {port}: {e}",
                binary.display()
            )
        })?;

    let ready = wait_for_health(port);
    let early_exit = child.try_wait().ok().flatten();
    if !ready {
        let _ = child.kill();
        let _ = child.wait();
        let code = early_exit.and_then(|s| s.code());
        anyhow::bail!("wm-server did not become ready on port {port} (exit {code:?})");
    }

    let token = read_token(root)?;
    Ok(DaemonClient {
        base_url,
        token: Arc::new(Mutex::new(Some(token))),
        token_path: mcp_token_path(root),
        child: Some(child),
    })
}

/// Read the port recorded in `.wm/server.json`, if any.
fn recorded_port(root: &Path) -> Option<u16> {
    let path = root.join(WM_DIR).join("server.json");
    let content = std::fs::read_to_string(path).ok()?;
    let value: Value = serde_json::from_str(&content).ok()?;
    value.get("port").and_then(Value::as_u64).map(|p| p as u16)
}

pub(crate) fn mcp_token_path(root: &Path) -> PathBuf {
    root.join(WM_DIR).join(STATE_DIR).join(MCP_TOKEN_FILE)
}

fn read_token(root: &Path) -> anyhow::Result<String> {
    let path = mcp_token_path(root);
    std::fs::read_to_string(&path)
        .map(|s| s.trim().to_string())
        .map_err(|e| anyhow::anyhow!("failed to read MCP token at {}: {e}", path.display()))
}

/// Poll `GET /api/health` until the daemon answers with a 2xx or the deadline
/// passes.
fn wait_for_health(port: u16) -> bool {
    let deadline = Instant::now() + Duration::from_secs(READY_DEADLINE_SECS);
    loop {
        if let Some(code) = crate::http_status(port, "/api/health") {
            if (200..300).contains(&code) {
                return true;
            }
        }
        if Instant::now() >= deadline {
            return false;
        }
        std::thread::sleep(Duration::from_millis(PROBE_INTERVAL_MS));
    }
}

/// POST a JSON payload, returning `(status, body)`. HTTP error statuses (>=400)
/// are returned as `Ok((status, body))` so the caller can implement token
/// retry; only transport failures are `Err` (as a message string).
pub(crate) fn post_json(
    base: &str,
    path: &str,
    payload: &Value,
    token: Option<&str>,
) -> Result<(u16, String), String> {
    let url = format!("{base}{path}");
    let mut request = ureq::post(&url)
        .set("content-type", "application/json")
        .timeout(HTTP_TIMEOUT);
    if let Some(tok) = token {
        request = request.set(TOKEN_HEADER, tok);
    }
    match request.send_string(&payload.to_string()) {
        Ok(resp) => Ok((resp.status(), resp.into_string().unwrap_or_default())),
        Err(ureq::Error::Status(code, resp)) => {
            Ok((code, resp.into_string().unwrap_or_default()))
        }
        Err(e) => Err(e.to_string()),
    }
}

/// POST, retrying exactly once after re-reading the token file if the first
/// attempt is rejected with 401 (the daemon rotates tokens on restart).
pub(crate) fn post_json_with_retry(
    base: &str,
    path: &str,
    payload: &Value,
    token_cell: &Mutex<Option<String>>,
    token_path: &Path,
) -> Result<(u16, String), String> {
    let token = token_cell.lock().unwrap().clone();
    let mut response = post_json(base, path, payload, token.as_deref())?;
    if response.0 == 401 {
        let fresh = std::fs::read_to_string(token_path)
            .ok()
            .map(|s| s.trim().to_string());
        if let Some(ref fresh) = fresh {
            *token_cell.lock().unwrap() = Some(fresh.clone());
        }
        response = post_json(base, path, payload, fresh.as_deref())?;
    }
    Ok(response)
}

/// Resolve the daemon once per CLI process and cache it.
fn daemon_client() -> anyhow::Result<Arc<DaemonClient>> {
    static DAEMON: OnceLock<anyhow::Result<Arc<DaemonClient>>> = OnceLock::new();
    let root = wm_core::config::detect_project_root()
        .ok_or_else(|| anyhow::anyhow!("No wiki-mem project found. Run 'wm init' first."))?;
    match DAEMON.get_or_init(|| detect_server_url(&root).map(Arc::new)) {
        Ok(arc) => Ok(Arc::clone(arc)),
        Err(e) => Err(anyhow::anyhow!("{e:#}")),
    }
}

/// Synchronous entry point for migrated CLI commands.
///
/// Calls the privileged MCP tool channel (`POST /api/mcp/tools/call`, body
/// `{name, arguments}`) with the MCP token. Returns the tool `data` on success
/// or an `anyhow` error carrying the daemon's `{code, error}` envelope.
pub(crate) fn call_tool(name: &str, arguments: Value) -> anyhow::Result<Value> {
    let daemon = daemon_client().map_err(|e| anyhow::anyhow!("{e:#}"))?;
    let payload = json!({ "name": name, "arguments": arguments });

    let (status, body) = post_json_with_retry(
        &daemon.base_url,
        MCP_TOOLS_CALL_PATH,
        &payload,
        &daemon.token,
        &daemon.token_path,
    )
    .map_err(|e| anyhow::anyhow!("wm-server unreachable: {e}"))?;
    if !(200..300).contains(&status) {
        return Err(anyhow::anyhow!(
            "wm-server rejected tools/call with HTTP {status}: {body}"
        ));
    }

    let parsed: Value = serde_json::from_str(&body)
        .map_err(|e| anyhow::anyhow!("invalid tools/call response: {e}"))?;
    if parsed.get("success").and_then(Value::as_bool) == Some(true) {
        Ok(parsed["data"].clone())
    } else {
        let error = parsed["error"]
            .as_str()
            .unwrap_or("tool error")
            .to_string();
        let code = parsed["code"].as_str().unwrap_or("TOOL_ERROR").to_string();
        Err(anyhow::anyhow!("[{code}] {error}"))
    }
}
