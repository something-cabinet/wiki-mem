//! HTTP-API E2E helper: spawns a real `wm-server` daemon on a fixture project
//! and drives its read-only web API (`/api/*`) over plain HTTP (ureq).
//!
//! Architecture (oracle D1): wm-server is the single daemon that owns the
//! engine, the read-only web HTTP API, and the Angular SPA. The MCP stdio
//! transport lives in wm-cli as a thin proxy; the web API is the only HTTP
//! surface of the daemon. This helper lets tests hit that surface directly.

use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use serde_json::Value;

const TOKEN_HEADER: &str = "x-wm-token";

/// The wm-cli binary next to the test binary (same target dir as wm-server).
fn get_binary_path() -> PathBuf {
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

fn get_server_binary_path() -> PathBuf {
    let mut path = get_binary_path();
    path.pop();
    let bin_name = if cfg!(windows) {
        "wm-server.exe"
    } else {
        "wm-server"
    };
    path.join(bin_name)
}

/// Bind an ephemeral port and return it (released on drop of the listener).
fn free_port() -> u16 {
    let listener = std::net::TcpListener::bind(("127.0.0.1", 0)).expect("bind ephemeral port");
    listener.local_addr().expect("local addr").port()
}

fn read_token(root: &std::path::Path, file: &str) -> String {
    let path = root.join(".wm").join("state").join(file);
    std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("failed to read {} at {}: {e}", file, path.display()))
        .trim()
        .to_string()
}

fn get(base: &str, path: &str, token: Option<&str>) -> (u16, String) {
    let mut request = ureq::get(&format!("{base}{path}"));
    if let Some(tok) = token {
        request = request.set(TOKEN_HEADER, tok);
    }
    match request.call() {
        Ok(resp) => (resp.status(), resp.into_string().unwrap_or_default()),
        Err(ureq::Error::Status(code, resp)) => (code, resp.into_string().unwrap_or_default()),
        Err(e) => (0, format!("transport error: {e}")),
    }
}

#[allow(dead_code, reason = "shared helper; only e2e_http uses header assertions")]
fn get_with_headers(base: &str, path: &str) -> (u16, Vec<(String, String)>, String) {
    match ureq::get(&format!("{base}{path}")).call() {
        Ok(resp) => {
            let status = resp.status();
            let headers = response_headers(&resp);
            let body = resp.into_string().unwrap_or_default();
            (status, headers, body)
        }
        Err(ureq::Error::Status(code, resp)) => {
            let headers = response_headers(&resp);
            let body = resp.into_string().unwrap_or_default();
            (code, headers, body)
        }
        Err(e) => (0, Vec::new(), format!("transport error: {e}")),
    }
}

#[allow(dead_code, reason = "shared helper; only e2e_http uses header assertions")]
fn response_headers(resp: &ureq::Response) -> Vec<(String, String)> {
    resp.headers_names()
        .into_iter()
        .filter_map(|name| {
            resp.header(&name)
                .map(|value| (name.to_ascii_lowercase(), value.to_string()))
        })
        .collect()
}

fn post(base: &str, path: &str, body: &Value, token: Option<&str>) -> (u16, String) {
    let mut request = ureq::post(&format!("{base}{path}")).set("content-type", "application/json");
    if let Some(tok) = token {
        request = request.set(TOKEN_HEADER, tok);
    }
    match request.send_string(&body.to_string()) {
        Ok(resp) => (resp.status(), resp.into_string().unwrap_or_default()),
        Err(ureq::Error::Status(code, resp)) => (code, resp.into_string().unwrap_or_default()),
        Err(e) => (0, format!("transport error: {e}")),
    }
}

/// A live `wm-server` daemon on a dedicated free port, with the web credential
/// read from the fixture project's `.wm/state/` after the health check passes.
/// Kills the child on drop so tests don't leak daemons.
pub struct DaemonHandle {
    child: Option<Child>,
    pub base_url: String,
    pub web_token: String,
}

impl DaemonHandle {
    /// Spawn `wm-server --port <free>` on `root` and wait until `/api/health`
    /// answers 2xx (bounded 30s). Panics with the daemon's early-exit status
    /// if it dies before becoming healthy.
    pub fn start(root: &std::path::Path) -> Self {
        Self::start_with_env(root, &[])
    }

    /// Like `start`, but injects extra environment variables into the daemon
    /// process (used to redirect `$HOME` so tests that touch the global layer
    /// — e.g. `wm_memory.promote` — never pollute the real home directory).
    pub fn start_with_env(root: &std::path::Path, envs: &[(&str, &str)]) -> Self {
        let port = free_port();
        let binary = get_server_binary_path();
        let mut cmd = Command::new(&binary);
        cmd.arg("--port")
            .arg(port.to_string())
            .current_dir(root)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        for (k, v) in envs {
            cmd.env(k, v);
        }
        let mut child = cmd.spawn().unwrap_or_else(|e| {
            panic!(
                "failed to spawn wm-server ({}) on port {port}: {e}",
                binary.display()
            )
        });

        let base_url = format!("http://127.0.0.1:{port}");
        let deadline = Instant::now() + Duration::from_secs(30);
        loop {
            let (status, _) = get(&base_url, "/api/health", None);
            if (200..300).contains(&status) {
                break;
            }
            if let Some(status) = child.try_wait().unwrap_or(None) {
                let _ = child.kill();
                panic!("wm-server exited early with {status:?} on port {port}");
            }
            assert!(
                Instant::now() < deadline,
                "wm-server did not become healthy within 30s on port {port}"
            );
            std::thread::sleep(Duration::from_millis(100));
        }

        Self {
            child: Some(child),
            base_url,
            web_token: read_token(root, "web-token"),
        }
    }

    /// Raw web-API POST with the read-only web token. (Used by e2e_http.)
    #[allow(dead_code, reason = "shared helper; not every test crate uses every verb")]
    pub fn web_post(&self, path: &str, body: &Value) -> (u16, Value) {
        let (status, body) = post(&self.base_url, path, body, Some(&self.web_token));
        (status, parse(body))
    }

    /// Raw request with an arbitrary credential (or none) for auth tests.
    pub fn raw(&self, method: &str, path: &str, body: &Value, token: Option<&str>) -> (u16, String) {
        match method {
            "GET" => get(&self.base_url, path, token),
            _ => post(&self.base_url, path, body, token),
        }
    }

    /// GET returning status, lowercased response headers, and body.
    #[allow(dead_code, reason = "shared helper; only e2e_http uses header assertions")]
    pub fn get_headers(&self, path: &str) -> (u16, Vec<(String, String)>, String) {
        get_with_headers(&self.base_url, path)
    }
}

fn parse(body: String) -> Value {
    serde_json::from_str(&body).unwrap_or_else(|e| {
        panic!("daemon returned invalid JSON ({e}): {body}")
    })
}

impl Drop for DaemonHandle {
    fn drop(&mut self) {
        if let Some(child) = &mut self.child {
            #[cfg(windows)]
            {
                let _ = std::process::Command::new("taskkill")
                    .args(["/F", "/T", "/PID", &child.id().to_string()])
                    .output();
            }
            #[cfg(not(windows))]
            {
                let _ = child.kill();
            }
            let _ = child.wait();
        }
    }
}
