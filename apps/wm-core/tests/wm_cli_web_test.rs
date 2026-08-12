//! `wm web` / `wm-server` HTTP contracts: port honoring, fail-fast on a taken
//! port, auth enforcement with audit lines, the removed proxy routes, SPA
//! fallback, and the singleton guard. The daemon is a real subprocess; HTTP is
//! driven with ureq.

#[path = "helpers/setup.rs"]
mod setup;
use setup::setup_test_project;

use std::io::Read;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use wm_constants::*;

fn wm_cli_path() -> PathBuf {
    if let Ok(p) = std::env::var("TEST_BINARY") {
        return PathBuf::from(p);
    }
    let exe = std::env::current_exe().expect("current exe");
    let mut path = exe.parent().unwrap();
    if path.ends_with("deps") {
        path = path.parent().unwrap();
    }
    let bin_name = if cfg!(windows) {
        "wm-cli.exe"
    } else {
        "wm-cli"
    };
    path.join(bin_name)
}

fn wm_server_path() -> PathBuf {
    let mut path = wm_cli_path();
    path.pop();
    let bin_name = if cfg!(windows) {
        "wm-server.exe"
    } else {
        "wm-server"
    };
    path.join(bin_name)
}

fn free_port() -> u16 {
    let listener = std::net::TcpListener::bind((LOCALHOST_ADDR, 0)).expect("bind ephemeral port");
    listener.local_addr().expect("local addr").port()
}

/// Web API token header, matching `wm-server`'s `web_token_service::header_name()`.
const WEB_TOKEN_HEADER: &str = "x-wm-token";

fn client() -> ureq::Agent {
    ureq::AgentBuilder::new()
        .timeout(Duration::from_secs(2))
        .build()
}

fn http_status(agent: &ureq::Agent, port: u16, path: &str) -> Option<u16> {
    let url = format!("http://{LOCALHOST_ADDR}:{port}{path}");
    match agent.get(&url).call() {
        Ok(resp) => Some(resp.status()),
        Err(ureq::Error::Status(code, _)) => Some(code),
        Err(_) => None,
    }
}

/// Issue a POST with an optional JSON body and optional token header. Returns
/// the HTTP status code and the response body.
fn http_post(
    agent: &ureq::Agent,
    port: u16,
    path: &str,
    body: &str,
    token: Option<&str>,
) -> Option<(u16, String)> {
    let url = format!("http://{LOCALHOST_ADDR}:{port}{path}");
    let mut request = agent.post(&url).set("content-type", "application/json");
    if let Some(tok) = token {
        request = request.set(WEB_TOKEN_HEADER, tok);
    }
    match request.send_string(body) {
        Ok(resp) => Some((resp.status(), resp.into_string().unwrap_or_default())),
        Err(ureq::Error::Status(code, resp)) => {
            Some((code, resp.into_string().unwrap_or_default()))
        }
        Err(e) => {
            eprintln!("transport error: {e}");
            None
        }
    }
}

/// Read the web API token persisted by wm-server at `.wm/state/web-token`.
fn read_web_token(root: &std::path::Path) -> String {
    let path = root.join(WM_DIR).join(STATE_DIR).join("web-token");
    std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("failed to read web token at {}: {e}", path.display()))
        .trim()
        .to_string()
}

fn drain<R: Read>(mut reader: R, buf: Arc<Mutex<Vec<u8>>>) {
    let mut tmp = [0u8; HTTP_PROBE_BUF_LEN];
    while let Ok(n) = reader.read(&mut tmp) {
        if n == 0 {
            break;
        }
        buf.lock().unwrap().extend_from_slice(&tmp[..n]);
    }
}

struct WebProcess {
    child: Child,
    stdout: Arc<Mutex<Vec<u8>>>,
    stderr: Arc<Mutex<Vec<u8>>>,
}

impl WebProcess {
    fn spawn(root: &std::path::Path, port: u16) -> WebProcess {
        let mut cmd = Command::new(wm_cli_path());
        cmd.args(["web", "--port", &port.to_string()]);
        cmd.current_dir(root);
        cmd.env("NO_COLOR", "1");
        cmd.env("WM_SERVER_PATH", wm_server_path());
        cmd.env_remove("WM_PROJECT");
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());
        #[cfg(unix)]
        {
            use std::os::unix::process::CommandExt;
            cmd.process_group(0);
        }
        let mut child = cmd.spawn().expect("spawn wm-cli web");
        let stdout = Arc::new(Mutex::new(Vec::new()));
        let stderr = Arc::new(Mutex::new(Vec::new()));
        if let Some(out) = child.stdout.take() {
            let buf = Arc::clone(&stdout);
            std::thread::spawn(move || drain(out, buf));
        }
        if let Some(err) = child.stderr.take() {
            let buf = Arc::clone(&stderr);
            std::thread::spawn(move || drain(err, buf));
        }
        WebProcess {
            child,
            stdout,
            stderr,
        }
    }

    fn output(&self) -> String {
        let out = self.stdout.lock().unwrap();
        let err = self.stderr.lock().unwrap();
        format!(
            "{}{}",
            String::from_utf8_lossy(&out),
            String::from_utf8_lossy(&err)
        )
    }

    /// Terminate the whole process group gracefully (wm-cli waits on the
    /// wm-server child, so the group must be signaled together).
    fn shutdown(&mut self) {
        #[cfg(windows)]
        {
            let _ = Command::new("taskkill")
                .args(["/F", "/T", "/PID", &self.child.id().to_string()])
                .output();
        }
        #[cfg(not(windows))]
        {
            let _ = Command::new("kill")
                .args(["-TERM", "--", &format!("-{}", self.child.id())])
                .output();
            let deadline = Instant::now() + Duration::from_secs(5);
            while self.child.try_wait().ok().flatten().is_none() && Instant::now() < deadline {
                std::thread::sleep(Duration::from_millis(50));
            }
        }
        let _ = self.child.wait();
    }
}

impl Drop for WebProcess {
    fn drop(&mut self) {
        self.shutdown();
    }
}

fn wait_for_health(proc: &mut WebProcess, port: u16) -> ureq::Agent {
    let agent = client();
    let deadline = Instant::now() + Duration::from_secs(READY_DEADLINE_SECS);
    loop {
        if let Some(code) = http_status(&agent, port, "/api/health") {
            if (200..300).contains(&code) {
                return agent;
            }
        }
        if let Ok(Some(status)) = proc.child.try_wait() {
            panic!(
                "wm-cli exited early ({status:?}) before server readiness. Output:\n{}",
                proc.output()
            );
        }
        assert!(
            Instant::now() < deadline,
            "server never became ready on port {port}. Output:\n{}",
            proc.output()
        );
        std::thread::sleep(Duration::from_millis(100));
    }
}

/// `wm web` serves the API on the requested port with a built SPA served at /.
#[test]
fn wm_cli_web_honors_port_flag() {
    let (_dir, root) = setup_test_project();
    let port = free_port();

    let mut proc = WebProcess::spawn(&root, port);
    let agent = wait_for_health(&mut proc, port);
    let status = http_status(&agent, port, "/api/health");
    assert_eq!(
        status,
        Some(200),
        "GET /api/health on requested port {port} should return 200. Output:\n{}",
        proc.output()
    );
    assert!(
        http_status(&agent, port, "/").is_some(),
        "GET / should respond (SPA served or Web UI not built). Output:\n{}",
        proc.output()
    );
    proc.shutdown();
}

/// A taken port must fail fast: wm web spawns wm-server, the bind fails, and
/// wm-cli exits reporting the failure — no silent fallback to another port.
#[test]
fn wm_cli_web_fails_fast_when_port_in_use() {
    let (_dir, root) = setup_test_project();
    let taken = free_port();
    let _stale = std::net::TcpListener::bind((LOCALHOST_ADDR, taken))
        .expect("pre-bind listener on chosen port");

    let mut proc = WebProcess::spawn(&root, taken);
    let deadline = Instant::now() + Duration::from_secs(READY_DEADLINE_SECS);
    loop {
        if proc.child.try_wait().ok().flatten().is_some() {
            break;
        }
        assert!(
            Instant::now() < deadline,
            "wm web must exit after wm-server fails to bind (no fallback retry). Output:\n{}",
            proc.output()
        );
        std::thread::sleep(Duration::from_millis(100));
    }

    let output = proc.output();
    assert!(
        output.contains("Address already in use"),
        "bind failure must surface in output:\n{output}"
    );
    assert!(
        output.contains("exited with code"),
        "wm-server failure must be reported by wm web:\n{output}"
    );
    proc.shutdown();
}

/// Without a built SPA, `wm web` must serve the API on the requested port and
/// 404 on GET / — API-only mode, with no false "web started" claims.
#[test]
fn wm_cli_web_serves_api_without_spa() {
    let (_dir, root) = setup_test_project();
    let port = free_port();

    let mut proc = WebProcess::spawn(&root, port);
    let agent = wait_for_health(&mut proc, port);
    let spa_status = http_status(&agent, port, "/");
    proc.shutdown();

    assert_eq!(
        spa_status,
        Some(404),
        "without a built SPA, GET / must 404 (API-only mode). Output:\n{}",
        proc.output()
    );
}

#[test]
fn wm_cli_web_api_rejects_unauthenticated_code_symbols() {
    let (_dir, root) = setup_test_project();
    let port = free_port();

    let mut proc = WebProcess::spawn(&root, port);
    let agent = wait_for_health(&mut proc, port);
    let (status, body) = http_post(&agent, port, "/api/code/symbols", "{}", None)
        .unwrap_or_else(|| panic!("POST /api/code/symbols should respond. Output:\n{}", proc.output()));
    proc.shutdown();

    assert_eq!(
        status,
        401,
        "unauthenticated POST to /api/code/symbols must be rejected. Body:\n{body}"
    );
}

/// Security regression (task wire-audit-sink-for-security-rejections): an
/// unauthenticated request to a protected route must leave a durable
/// `auth_failure` audit line in `<root>/.wm/log.jsonl`, naming the rejected
/// route (never the token).
#[test]
fn wm_cli_web_auth_failure_leaves_audit_line() {
    let (_dir, root) = setup_test_project();
    let port = free_port();

    let mut proc = WebProcess::spawn(&root, port);
    let agent = wait_for_health(&mut proc, port);
    let (status, body) = http_post(&agent, port, "/api/code/symbols", "{}", None)
        .unwrap_or_else(|| panic!("POST /api/code/symbols should respond. Output:\n{}", proc.output()));
    proc.shutdown();

    assert_eq!(
        status,
        401,
        "unauthenticated POST to /api/code/symbols must be rejected. Body:\n{body}"
    );

    let log_path = root.join(WM_DIR).join(LOG_FILE);
    let content = std::fs::read_to_string(&log_path)
        .unwrap_or_else(|e| panic!("audit log should exist at {}: {e}", log_path.display()));
    assert!(
        content.contains("\"kind\":\"auth_failure\""),
        "auth rejection must emit an auth_failure audit event, got: {}",
        content
    );
    assert!(
        content.contains("\"category\":\"security\""),
        "auth failure must be categorised as security, got: {}",
        content
    );
    assert!(
        content.contains("/api/code/symbols"),
        "audit line must name the rejected route (not the token), got: {}",
        content
    );
}

#[test]
fn wm_cli_web_api_code_symbols_authenticated() {
    let (_dir, root) = setup_test_project();
    let port = free_port();

    let mut proc = WebProcess::spawn(&root, port);
    let agent = wait_for_health(&mut proc, port);
    let token = read_web_token(&root);
    let (status, body) = http_post(&agent, port, "/api/code/symbols", "{}", Some(&token))
        .unwrap_or_else(|| panic!("POST /api/code/symbols should respond. Output:\n{}", proc.output()));
    proc.shutdown();

    assert_eq!(status, 200, "authenticated POST should succeed. Body:\n{body}");
    let parsed: serde_json::Value = serde_json::from_str(&body)
        .unwrap_or_else(|e| panic!("response should be valid JSON: {e}. Body:\n{body}"));
    assert_eq!(
        parsed["success"],
        serde_json::json!(true),
        "response should carry success envelope. Body:\n{body}"
    );
    assert!(
        parsed["data"].get("symbols").is_some(),
        "response should carry data.symbols. Body:\n{body}"
    );
}

#[test]
fn wm_cli_web_api_old_tools_route_removed() {
    let (_dir, root) = setup_test_project();
    let port = free_port();

    let mut proc = WebProcess::spawn(&root, port);
    let agent = wait_for_health(&mut proc, port);
    let token = read_web_token(&root);
    let (status, body) = http_post(&agent, port, "/api/tools/wm_code.symbols", "{}", Some(&token))
        .unwrap_or_else(|| {
            panic!(
                "POST /api/tools/wm_code.symbols should respond. Output:\n{}",
                proc.output()
            )
        });
    proc.shutdown();

    assert_eq!(
        status, 404,
        "the generic /api/tools/{{name}} dispatch route must be gone. Body:\n{body}"
    );
}

/// The privileged MCP proxy channel is gone: `/api/mcp/*` must 404, and the
/// daemon writes only the web credential (no `mcp-token` file).
#[test]
fn wm_server_mcp_channel_removed() {
    let (_dir, root) = setup_test_project();
    let port = free_port();

    let mut proc = WebProcess::spawn(&root, port);
    let agent = wait_for_health(&mut proc, port);
    let token = read_web_token(&root);
    let (status, body) = http_post(&agent, port, "/api/mcp/tools/list", "{}", Some(&token))
        .unwrap_or_else(|| panic!("POST /api/mcp/tools/list should respond. Output:\n{}", proc.output()));
    proc.shutdown();

    assert_eq!(status, 404, "the /api/mcp/* channel must be gone. Body:\n{body}");
    assert!(
        !root.join(WM_DIR).join(STATE_DIR).join("mcp-token").exists(),
        "the daemon must not write a mcp-token credential"
    );
}

/// Singleton guard (AC-0): a second wm-server on the same port refuses to start
/// when a live daemon holds `.wm/server.json`.
#[test]
fn wm_server_singleton_refuses_duplicate() {
    let (_dir, root) = setup_test_project();
    let port = free_port();

    let mut cmd = Command::new(wm_server_path());
    cmd.arg("--port")
        .arg(port.to_string())
        .current_dir(&root)
        .env_remove("WM_PROJECT")
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        cmd.process_group(0);
    }
    let mut first = cmd.spawn().expect("spawn first wm-server");

    let agent = client();
    let deadline = Instant::now() + Duration::from_secs(READY_DEADLINE_SECS);
    while http_status(&agent, port, "/api/health") != Some(200) {
        assert!(
            Instant::now() < deadline,
            "first wm-server never became ready"
        );
        std::thread::sleep(Duration::from_millis(100));
    }

    // Second instance must exit non-zero with a clear singleton message.
    let out = Command::new(wm_server_path())
        .arg("--port")
        .arg(port.to_string())
        .current_dir(&root)
        .env_remove("WM_PROJECT")
        .output()
        .expect("spawn second wm-server");
    assert!(
        !out.status.success(),
        "second wm-server must refuse to start"
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("already running"),
        "expected singleton message, got stderr: {stderr}"
    );

    // The first daemon is still healthy and serving.
    assert_eq!(http_status(&agent, port, "/api/health"), Some(200));

    terminate_group(&mut first);
}

/// Signal the whole process group and wait for it to exit.
fn terminate_group(child: &mut Child) {
    #[cfg(windows)]
    {
        let _ = Command::new("taskkill")
            .args(["/F", "/T", "/PID", &child.id().to_string()])
            .output();
    }
    #[cfg(not(windows))]
    {
        let _ = Command::new("kill")
            .args(["-TERM", "--", &format!("-{}", child.id())])
            .output();
        let deadline = Instant::now() + Duration::from_secs(5);
        while child.try_wait().ok().flatten().is_none() && Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(50));
        }
    }
    let _ = child.wait();
}
