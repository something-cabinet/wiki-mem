#[path = "helpers/setup.rs"]
mod setup;
use setup::setup_test_project;

use std::io::{BufRead, BufReader, Read, Write};
use std::path::PathBuf;
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
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

fn http_status(port: u16, path: &str) -> Option<u16> {
    use std::io::Write;
    use std::net::TcpStream;

    let addr = format!("{LOCALHOST_ADDR}:{port}");
    let Ok(mut stream) = TcpStream::connect(&addr) else {
        return None;
    };
    let _ = stream.set_read_timeout(Some(Duration::from_secs(HTTP_PROBE_READ_TIMEOUT_SECS)));
    let request = format!(
        "GET {path} HTTP/1.1\r\nHost: {LOCALHOST_ADDR}:{port}\r\nConnection: close\r\n\r\n"
    );
    if stream.write_all(request.as_bytes()).is_err() {
        return None;
    }
    let mut buf = [0u8; HTTP_PROBE_BUF_LEN];
    let mut filled = 0;
    while filled < buf.len() {
        let n = match stream.read(&mut buf[filled..]) {
            Ok(n) => n,
            Err(_) => break,
        };
        if n == 0 {
            break;
        }
        filled += n;
        if buf[..filled].contains(&b'\n') {
            break;
        }
    }
    let head = String::from_utf8_lossy(&buf[..filled]);
    let mut parts = head.lines().next()?.split_whitespace();
    parts.nth(1)?.parse().ok()
}

/// Web API token header, matching `wm-server`'s `web_token_service::header_name()`.
const WEB_TOKEN_HEADER: &str = "x-wm-token";

/// Issue a POST with an optional JSON body and optional token header. Returns
/// the HTTP status code and the response body.
fn http_post(port: u16, path: &str, body: &str, token: Option<&str>) -> Option<(u16, String)> {
    use std::io::Write;
    use std::net::TcpStream;

    let addr = format!("{LOCALHOST_ADDR}:{port}");
    let mut stream = TcpStream::connect(&addr).ok()?;
    stream
        .set_read_timeout(Some(Duration::from_secs(HTTP_PROBE_READ_TIMEOUT_SECS)))
        .ok()?;

    let mut request = format!(
        "POST {path} HTTP/1.1\r\nHost: {LOCALHOST_ADDR}:{port}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n",
        body.len()
    );
    if let Some(tok) = token {
        request.push_str(&format!("{WEB_TOKEN_HEADER}: {tok}\r\n"));
    }
    request.push_str("\r\n");
    request.push_str(body);

    if stream.write_all(request.as_bytes()).is_err() {
        return None;
    }

    let mut raw = Vec::new();
    let mut tmp = [0u8; HTTP_PROBE_BUF_LEN];
    loop {
        match stream.read(&mut tmp) {
            Ok(0) | Err(_) => break,
            Ok(n) => raw.extend_from_slice(&tmp[..n]),
        }
    }

    let text = String::from_utf8_lossy(&raw);
    let status = text.lines().next()?.split_whitespace().nth(1)?.parse().ok()?;
    let body = match text.find("\r\n\r\n") {
        Some(idx) => text[idx + 4..].to_string(),
        None => String::new(),
    };
    Some((status, body))
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

    fn kill_group(&mut self) {
        #[cfg(windows)]
        {
            let _ = Command::new("taskkill")
                .args(["/F", "/T", "/PID", &self.child.id().to_string()])
                .output();
        }
        #[cfg(not(windows))]
        {
            let _ = Command::new("kill")
                .args(["-9", "--", &format!("-{}", self.child.id())])
                .output();
        }
        let _ = self.child.wait();
    }
}

impl Drop for WebProcess {
    fn drop(&mut self) {
        self.kill_group();
    }
}

fn wait_for_health(proc: &mut WebProcess, port: u16) {
    let deadline = Instant::now() + Duration::from_secs(READY_DEADLINE_SECS);
    loop {
        if let Some(code) = http_status(port, "/api/health") {
            if (200..300).contains(&code) {
                return;
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

fn wait_for_output_containing(proc: &WebProcess, needles: &[&str]) -> String {
    let deadline = Instant::now() + Duration::from_secs(READY_DEADLINE_SECS);
    loop {
        let output = proc.output();
        if needles.iter().all(|needle| output.contains(needle)) {
            return output;
        }
        assert!(
            Instant::now() < deadline,
            "expected lines {needles:?} missing from output:\n{output}"
        );
        std::thread::sleep(Duration::from_millis(100));
    }
}

fn assert_lifecycle_order(output: &str) {
    let lines = [
        "Starting wm-server",
        "wm-server started",
        "Starting wm-web",
        "wm-web started",
    ];
    let mut cursor = 0;
    for line in lines {
        let pos = match output.find(line) {
            Some(pos) => pos,
            None => panic!("missing lifecycle line {line:?} in output:\n{output}"),
        };
        assert!(
            pos >= cursor,
            "lifecycle line {line:?} appears out of order:\n{output}"
        );
        cursor = pos + line.len();
    }
}

fn create_fake_spa(root: &std::path::Path) {
    let dir = root
        .join("apps")
        .join("wm-web")
        .join("dist")
        .join("browser");
    std::fs::create_dir_all(&dir).expect("create fake spa dir");
    std::fs::write(dir.join("index.html"), "<html></html>").expect("write fake spa index");
}

#[test]
fn wm_cli_web_lifecycle_logs_in_order() {
    let (_dir, root) = setup_test_project();
    create_fake_spa(&root);
    let port = free_port();

    let mut proc = WebProcess::spawn(&root, port);
    wait_for_health(&mut proc, port);
    assert_eq!(
        http_status(port, "/"),
        Some(200),
        "fake SPA should be served on port {port}. Output:\n{}",
        proc.output()
    );
    let output = wait_for_output_containing(
        &proc,
        &[
            "Starting wm-server",
            "wm-server started",
            "Starting wm-web",
            "wm-web started",
        ],
    );
    proc.kill_group();

    assert_lifecycle_order(&output);
}

#[test]
fn wm_cli_web_honors_port_flag() {
    let (_dir, root) = setup_test_project();
    let port = free_port();

    let mut proc = WebProcess::spawn(&root, port);
    wait_for_health(&mut proc, port);
    let status = http_status(port, "/api/health");
    assert_eq!(
        status,
        Some(200),
        "GET /api/health on requested port {port} should return 200. Output:\n{}",
        proc.output()
    );
    let root_status = http_status(port, "/");
    assert!(
        root_status.is_some(),
        "GET / on requested port {port} should respond (SPA served or Web UI not built). Output:\n{}",
        proc.output()
    );
    proc.kill_group();
}

fn fallback_port(output: &str, occupied: u16) -> u16 {
    let marker = "Starting wm-server on port ";
    let start = output
        .find(marker)
        .expect("Starting wm-server line present");
    let digits: String = output[start + marker.len()..]
        .chars()
        .take_while(|c| c.is_ascii_digit())
        .collect();
    let port = digits
        .parse()
        .expect("parse port from Starting wm-server line");
    assert_ne!(
        port, occupied,
        "must not start the server on the occupied port:\n{output}"
    );
    port
}

#[test]
fn wm_cli_web_falls_back_when_port_in_use() {
    let (_dir, root) = setup_test_project();
    create_fake_spa(&root);
    let taken = free_port();
    let _stale = std::net::TcpListener::bind((LOCALHOST_ADDR, taken))
        .expect("pre-bind listener on chosen port");

    let mut proc = WebProcess::spawn(&root, taken);
    let in_use_note = format!("port {taken} in use");
    let output = wait_for_output_containing(
        &proc,
        &[
            in_use_note.as_str(),
            "Starting wm-server",
            "wm-server started",
            "Starting wm-web",
            "wm-web started",
        ],
    );
    let fallback = fallback_port(&output, taken);
    assert!(
        http_status(fallback, "/api/health").is_some_and(|c| (200..300).contains(&c)),
        "fallback server should serve on port {fallback}. Output:\n{output}"
    );
    proc.kill_group();

    let never_spawn_on_taken = format!("Starting wm-server on port {taken}");
    assert!(
        !output.contains(&never_spawn_on_taken),
        "must not spawn on the occupied port:\n{output}"
    );
    assert_lifecycle_order(&output);
}

#[test]
fn wm_cli_web_logs_not_built_without_started() {
    let (_dir, root) = setup_test_project();
    let port = free_port();

    let mut proc = WebProcess::spawn(&root, port);
    wait_for_health(&mut proc, port);
    let output = wait_for_output_containing(
        &proc,
        &[
            "Starting wm-web",
            "Web UI not built (GET / returned 404); wm-server serving API only",
        ],
    );
    proc.kill_group();

    assert!(
        !output.contains("wm-web started"),
        "must not claim wm-web started when the SPA is not built:\n{output}"
    );
    let start = output
        .find("Starting wm-web")
        .expect("Starting wm-web present");
    let note = output
        .find("Web UI not built")
        .expect("not-built note present");
    assert!(
        start < note,
        "Starting wm-web must precede the not-built note:\n{output}"
    );
}

#[test]
fn wm_cli_web_api_rejects_unauthenticated_code_symbols() {
    let (_dir, root) = setup_test_project();
    let port = free_port();

    let mut proc = WebProcess::spawn(&root, port);
    wait_for_health(&mut proc, port);
    let (status, body) = http_post(port, "/api/code/symbols", "{}", None)
        .unwrap_or_else(|| panic!("POST /api/code/symbols should respond. Output:\n{}", proc.output()));
    proc.kill_group();

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
    wait_for_health(&mut proc, port);
    let (status, body) = http_post(port, "/api/code/symbols", "{}", None)
        .unwrap_or_else(|| panic!("POST /api/code/symbols should respond. Output:\n{}", proc.output()));
    proc.kill_group();

    assert_eq!(
        status,
        401,
        "unauthenticated POST to /api/code/symbols must be rejected. Body:\n{body}"
    );

    // The rejection must have been persisted to the shared audit sink.
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
    wait_for_health(&mut proc, port);
    let token = read_web_token(&root);
    let (status, body) = http_post(port, "/api/code/symbols", "{}", Some(&token)).unwrap_or_else(
        || panic!("POST /api/code/symbols should respond. Output:\n{}", proc.output()),
    );
    proc.kill_group();

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
    wait_for_health(&mut proc, port);
    let token = read_web_token(&root);
    let (status, body) = http_post(port, "/api/tools/wm_code.symbols", "{}", Some(&token))
        .unwrap_or_else(|| {
            panic!(
                "POST /api/tools/wm_code.symbols should respond. Output:\n{}",
                proc.output()
            )
        });
    proc.kill_group();

    assert_eq!(
        status, 404,
        "the generic /api/tools/{{name}} dispatch route must be gone. Body:\n{body}"
    );
}

/// Read the MCP proxy token persisted by wm-server at `.wm/state/mcp-token`.
fn read_mcp_token(root: &std::path::Path) -> String {
    let path = root.join(WM_DIR).join(STATE_DIR).join("mcp-token");
    std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("failed to read MCP token at {}: {e}", path.display()))
        .trim()
        .to_string()
}

/// Cross-token isolation for the privileged MCP channel (AC-2):
/// - web token → 401 on /api/mcp/*
/// - mcp token → 401 on the read-only web API (/api/pages/list)
/// - /api/health stays token-free
/// - D2 regression: /api/tools/{name} still 404s
#[test]
fn wm_server_mcp_channel_cross_token_isolation() {
    let (_dir, root) = setup_test_project();
    let port = free_port();

    let mut proc = WebProcess::spawn(&root, port);
    wait_for_health(&mut proc, port);
    let web_token = read_web_token(&root);
    let mcp_token = read_mcp_token(&root);
    assert_ne!(
        web_token, mcp_token,
        "web and MCP tokens must be distinct credentials"
    );

    // Web token must NOT authorize the MCP channel.
    let (status, body) = http_post(port, "/api/mcp/tools/list", "{}", Some(&web_token))
        .unwrap_or_else(|| panic!("POST /api/mcp/tools/list should respond. Output:\n{}", proc.output()));
    assert_eq!(status, 401, "web token must be rejected on /api/mcp/tools/list. Body:\n{body}");

    let (status, body) = http_post(
        port,
        "/api/mcp/tools/call",
        r#"{"name":"wm_search.query","arguments":{"q":"x"}}"#,
        Some(&web_token),
    )
    .unwrap_or_else(|| panic!("POST /api/mcp/tools/call should respond. Output:\n{}", proc.output()));
    assert_eq!(status, 401, "web token must be rejected on /api/mcp/tools/call. Body:\n{body}");

    // MCP token must NOT authorize the read-only web API.
    let (status, body) = http_post(port, "/api/pages/list", "{}", Some(&mcp_token))
        .unwrap_or_else(|| panic!("POST /api/pages/list should respond. Output:\n{}", proc.output()));
    assert_eq!(status, 401, "mcp token must be rejected on /api/pages/list. Body:\n{body}");

    // Health stays token-free.
    assert_eq!(http_status(port, "/api/health"), Some(200));

    // D2 regression: generic dispatch route is gone.
    let (status, body) = http_post(port, "/api/tools/wm_code.symbols", "{}", Some(&web_token))
        .unwrap_or_else(|| {
            panic!(
                "POST /api/tools/wm_code.symbols should respond. Output:\n{}",
                proc.output()
            )
        });
    assert_eq!(status, 404, "D2 regression: /api/tools/{{name}} must still 404. Body:\n{body}");

    // Positive path: the MCP token authorizes the MCP channel.
    let (status, body) = http_post(port, "/api/mcp/tools/list", "{}", Some(&mcp_token))
        .unwrap_or_else(|| panic!("POST /api/mcp/tools/list should respond. Output:\n{}", proc.output()));
    proc.kill_group();
    assert_eq!(status, 200, "mcp token should authorize /api/mcp/tools/list. Body:\n{body}");
    let parsed: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(parsed["success"], serde_json::json!(true));
    assert!(parsed["data"]["tools"].as_array().is_some_and(|t| !t.is_empty()));
}

/// Minimal stdio JSON-RPC client driving a spawned `wm-cli mcp` proxy.
struct McpProcess {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
    next_id: u64,
    stderr: Arc<Mutex<Vec<u8>>>,
}

impl McpProcess {
    fn spawn(root: &std::path::Path) -> McpProcess {
        let mut cmd = Command::new(wm_cli_path());
        cmd.arg("mcp");
        cmd.current_dir(root);
        cmd.env("NO_COLOR", "1");
        cmd.env_remove("WM_PROJECT");
        cmd.stdin(Stdio::piped());
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());
        #[cfg(unix)]
        {
            use std::os::unix::process::CommandExt;
            cmd.process_group(0);
        }
        let mut child = cmd.spawn().expect("spawn wm-cli mcp");
        let stdin = child.stdin.take().expect("mcp stdin");
        let stdout = child.stdout.take().expect("mcp stdout");
        let stderr = Arc::new(Mutex::new(Vec::new()));
        if let Some(err) = child.stderr.take() {
            let buf = Arc::clone(&stderr);
            std::thread::spawn(move || drain(err, buf));
        }
        McpProcess {
            child,
            stdin,
            stdout: BufReader::new(stdout),
            next_id: 1,
            stderr,
        }
    }

    /// Send a JSON-RPC request and read the response with matching id.
    fn request(&mut self, method: &str, params: serde_json::Value) -> serde_json::Value {
        let id = self.next_id;
        self.next_id += 1;
        let msg = serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": method,
            "params": params,
        });
        writeln!(self.stdin, "{msg}").expect("write mcp request");
        self.stdin.flush().expect("flush mcp request");
        read_mcp_response(&mut self.stdout, id)
    }

    /// Perform the MCP initialize handshake, then the initialized notification.
    fn initialize(&mut self) {
        let resp = self.request(
            "initialize",
            serde_json::json!({
                "protocolVersion": "2025-11-25",
                "capabilities": {},
                "clientInfo": { "name": "wm-proxy-test", "version": "0.1.0" },
            }),
        );
        assert!(
            resp.get("result").is_some(),
            "initialize handshake failed: {resp}\nstderr:\n{}",
            String::from_utf8_lossy(&self.stderr.lock().unwrap())
        );
        let notified = serde_json::json!({"jsonrpc": "2.0", "method": "notifications/initialized"});
        writeln!(self.stdin, "{notified}").expect("write initialized notification");
        self.stdin.flush().expect("flush initialized notification");
    }

    fn call_tool(&mut self, name: &str, arguments: serde_json::Value) -> serde_json::Value {
        self.request(
            "tools/call",
            serde_json::json!({ "name": name, "arguments": arguments }),
        )
    }

    fn kill(&mut self) {
        #[cfg(windows)]
        {
            let _ = Command::new("taskkill")
                .args(["/F", "/T", "/PID", &self.child.id().to_string()])
                .output();
        }
        #[cfg(not(windows))]
        {
            let _ = Command::new("kill")
                .args(["-9", "--", &format!("-{}", self.child.id())])
                .output();
        }
        let _ = self.child.wait();
    }
}

fn read_mcp_response(reader: &mut BufReader<ChildStdout>, id: u64) -> serde_json::Value {
    let deadline = Instant::now() + Duration::from_secs(READY_DEADLINE_SECS);
    let mut line = String::new();
    loop {
        line.clear();
        let n = reader.read_line(&mut line).expect("read mcp response line");
        assert!(
            n > 0,
            "wm-cli mcp closed stdout before responding to id {id}"
        );
        if let Ok(value) = serde_json::from_str::<serde_json::Value>(&line) {
            if value.get("id").and_then(serde_json::Value::as_u64) == Some(id) {
                return value;
            }
        }
        assert!(
            Instant::now() < deadline,
            "timed out waiting for MCP response id {id}; last line: {line}"
        );
    }
}

fn tool_result_text(resp: &serde_json::Value) -> String {
    resp["result"]["content"][0]["text"]
        .as_str()
        .unwrap_or_default()
        .to_string()
}

fn tool_is_error(resp: &serde_json::Value) -> bool {
    resp["result"]["isError"].as_bool() == Some(true)
}

/// Kill the daemon recorded in `.wm/server.json` (spawned by the proxy).
fn kill_recorded_daemon(root: &std::path::Path) {
    let path = root.join(WM_DIR).join("server.json");
    let Ok(content) = std::fs::read_to_string(&path) else {
        return;
    };
    let Ok(value) = serde_json::from_str::<serde_json::Value>(&content) else {
        return;
    };
    if let Some(pid) = value.get("pid").and_then(serde_json::Value::as_u64) {
        let _ = Command::new("kill").arg("-9").arg(pid.to_string()).output();
    }
}

/// End-to-end MCP proxy test (AC-3/4/5): spawn the proxy with no daemon up,
/// let it discover+spawn wm-server in parallel with the handshake, then assert
/// dynamic tool discovery, a search call, and a page create→get→delete
/// round-trip through a real MCP client.
#[test]
fn wm_mcp_proxy_tools_and_calls_roundtrip() {
    let (_dir, root) = setup_test_project();
    let port = free_port();

    // Record a port for the proxy to spawn the daemon on. The stale pid forces
    // the discovery path to health-check (down) and spawn a fresh daemon.
    let server_json = root.join(WM_DIR).join("server.json");
    std::fs::create_dir_all(server_json.parent().unwrap()).unwrap();
    std::fs::write(
        &server_json,
        serde_json::to_string(&serde_json::json!({
            "port": port,
            "pid": 999999,
            "started_at": "stale"
        }))
        .unwrap(),
    )
    .unwrap();

    let mut proc = McpProcess::spawn(&root);
    proc.initialize();

    // Dynamic tool discovery through the proxy (no STATIC_TOOLS).
    let list = proc.request("tools/list", serde_json::json!({}));
    let list_result = list
        .get("result")
        .unwrap_or_else(|| panic!("tools/list failed: {list}"));
    let tools = list_result["tools"].as_array().expect("tools array");
    assert!(
        tools.len() > 10,
        "expected many tools from dynamic discovery, got {}: {list}",
        tools.len()
    );

    // The proxy spawned the daemon on `port`. Compare the proxy's tool list
    // against the daemon registry (tool count must match).
    let mcp_token = read_mcp_token(&root);
    let (status, body) = http_post(port, "/api/mcp/tools/list", "{}", Some(&mcp_token))
        .unwrap_or_else(|| panic!("daemon /api/mcp/tools/list should respond"));
    assert_eq!(status, 200, "daemon list endpoint: {body}");
    let daemon: serde_json::Value = serde_json::from_str(&body).unwrap();
    let daemon_tools = daemon["data"]["tools"].as_array().unwrap();
    assert_eq!(
        tools.len(),
        daemon_tools.len(),
        "proxy tool count must equal daemon registry count"
    );

    // wm_search.query works through the proxy.
    let search = proc.call_tool(
        "wm_search.query",
        serde_json::json!({ "q": "proxy", "limit": 5 }),
    );
    assert!(!tool_is_error(&search), "wm_search.query failed: {search}");
    let search_text = tool_result_text(&search);
    let parsed_search: serde_json::Value =
        serde_json::from_str(&search_text).unwrap_or(serde_json::Value::Null);
    assert!(
        parsed_search.get("results").is_some(),
        "search results missing: {search_text}"
    );

    // wm_page create → get → delete round-trip.
    let create = proc.call_tool(
        "wm_page",
        serde_json::json!({
            "action": "create",
            "path": "concepts/proxy-test",
            "title": "Proxy Test",
            "content": "# Proxy Test\n\nCreated through the MCP proxy.",
        }),
    );
    assert!(!tool_is_error(&create), "wm_page create failed: {create}");
    let create_text = tool_result_text(&create);
    let created: serde_json::Value =
        serde_json::from_str(&create_text).unwrap_or(serde_json::Value::Null);
    let page_id = created["id"]
        .as_str()
        .expect("created page id")
        .to_string();

    let get = proc.call_tool("wm_page", serde_json::json!({ "action": "get", "id": page_id }));
    assert!(!tool_is_error(&get), "wm_page get failed: {get}");
    let get_text = tool_result_text(&get);
    assert!(
        get_text.contains("Proxy Test"),
        "get should return the created page: {get_text}"
    );

    let delete = proc.call_tool(
        "wm_page",
        serde_json::json!({ "action": "delete", "id": page_id }),
    );
    assert!(!tool_is_error(&delete), "wm_page delete failed: {delete}");

    proc.kill();
    kill_recorded_daemon(&root);
}

/// Migrated CLI commands (task b78584) talk to the wm-server daemon over the
/// privileged MCP channel. This test spawns a daemon directly, then drives
/// `wm-cli search` and `wm-cli page get` against it (the CLI health-checks the
/// recorded port and reuses the live daemon — no second spawn).
#[test]
fn wm_cli_http_commands_against_live_daemon() {
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
    let mut daemon = cmd.spawn().expect("spawn wm-server");

    let deadline = Instant::now() + Duration::from_secs(READY_DEADLINE_SECS);
    while http_status(port, "/api/health") != Some(200) {
        assert!(
            Instant::now() < deadline,
            "wm-server never became ready on port {port}"
        );
        std::thread::sleep(Duration::from_millis(100));
    }

    // Seed a page through the daemon's MCP channel, then read it back through
    // the CLI (which must reuse the live daemon).
    let mcp_token = read_mcp_token(&root);
    let (status, body) = http_post(
        port,
        "/api/mcp/tools/call",
        r##"{"name":"wm_page","arguments":{"action":"create","path":"concepts/http-get","title":"HTTP Get","content":"# HTTP Get\n\nCreated via daemon."}}"##,
        Some(&mcp_token),
    )
    .unwrap_or_else(|| panic!("daemon tools/call should respond"));
    assert_eq!(status, 200, "seed page create should succeed: {body}");

    let run = |args: &[&str]| {
        Command::new(wm_cli_path())
            .args(args)
            .current_dir(&root)
            .env("NO_COLOR", "1")
            .env("WM_SERVER_PATH", wm_server_path())
            .env_remove("WM_PROJECT")
            .output()
            .expect("run wm-cli")
    };

    // wm_cli page get — migrated to HTTP.
    let get = run(&["page", "get", "wiki:concepts:http-get", "--json"]);
    assert!(
        get.status.success(),
        "page get failed: {}",
        String::from_utf8_lossy(&get.stderr)
    );
    let parsed_get: serde_json::Value = serde_json::from_slice(&get.stdout).unwrap_or_else(|e| {
        panic!("page get output should be JSON: {e}\n{}", String::from_utf8_lossy(&get.stdout))
    });
    let content = parsed_get["content"].as_str().unwrap_or("");
    assert!(
        content.contains("HTTP Get"),
        "page get should return the seeded page: {content}"
    );

    // wm_cli search query — migrated to HTTP.
    let search = run(&["search", "query", "HTTP Get", "--json"]);
    assert!(
        search.status.success(),
        "search failed: {}",
        String::from_utf8_lossy(&search.stderr)
    );
    let parsed_search: serde_json::Value = serde_json::from_slice(&search.stdout).unwrap_or_else(
        |e| {
            panic!(
                "search output should be JSON: {e}\n{}",
                String::from_utf8_lossy(&search.stdout)
            )
        },
    );
    let results = parsed_search
        .get("results")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    assert!(
        results
            .iter()
            .any(|r| r["id"].as_str().unwrap_or("").starts_with("wiki:concepts:http-get")),
        "search should find the seeded page: {parsed_search}"
    );

    #[cfg(unix)]
    let _ = Command::new("kill")
        .args(["-9", "--", &format!("-{}", daemon.id())])
        .output();
    #[cfg(windows)]
    let _ = Command::new("taskkill")
        .args(["/F", "/T", "/PID", &daemon.id().to_string()])
        .output();
    let _ = daemon.wait();
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

    let deadline = Instant::now() + Duration::from_secs(READY_DEADLINE_SECS);
    while http_status(port, "/api/health") != Some(200) {
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
    assert_eq!(http_status(port, "/api/health"), Some(200));

    #[cfg(unix)]
    let _ = Command::new("kill")
        .args(["-9", "--", &format!("-{}", first.id())])
        .output();
    #[cfg(windows)]
    let _ = Command::new("taskkill")
        .args(["/F", "/T", "/PID", &first.id().to_string()])
        .output();
    let _ = first.wait();
}
