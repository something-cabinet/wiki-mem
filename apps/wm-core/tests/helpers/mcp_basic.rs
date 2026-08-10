use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::time::Duration;

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

/// Bind an ephemeral port and return it (released on drop of the listener).
fn free_port() -> u16 {
    let listener = std::net::TcpListener::bind(("127.0.0.1", 0)).expect("bind ephemeral port");
    listener.local_addr().expect("local addr").port()
}

/// Kill the daemon recorded in `.wm/server.json` (spawned by the proxy).
fn kill_recorded_daemon(project_dir: &std::path::Path) {
    let path = project_dir.join(".wm").join("server.json");
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

pub struct MCPClient {
    child: Child,
    stdin: std::io::BufWriter<std::process::ChildStdin>,
    reader: BufReader<std::process::ChildStdout>,
    next_id: u64,
    project_dir: std::path::PathBuf,
}

impl MCPClient {
    pub fn start(project_dir: &std::path::Path) -> Self {
        let bin = get_binary_path();
        // `wm mcp` is now a stdio→HTTP proxy (task #41): it discovers the
        // daemon port via `.wm/server.json` and spawns wm-server if nothing is
        // live. Point it at a dedicated free port so parallel tests don't race
        // on the default 4090.
        let port = free_port();
        let server_json = project_dir.join(".wm").join("server.json");
        std::fs::create_dir_all(server_json.parent().unwrap()).unwrap();
        std::fs::write(
            &server_json,
            serde_json::to_string(&serde_json::json!({
                "port": port,
                "pid": 0,
                "started_at": "e2e-test",
            }))
            .unwrap(),
        )
        .unwrap();

        let mut cmd = Command::new(&bin);
        cmd.arg("mcp");
        cmd.current_dir(project_dir);
        cmd.env("NO_COLOR", "1");
        cmd.env_remove("WM_PROJECT");
        cmd.stdin(Stdio::piped());
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::null());
        #[cfg(unix)]
        {
            use std::os::unix::process::CommandExt;
            cmd.process_group(0);
        }

        let mut child = cmd.spawn().expect("failed to spawn wm serve");
        let stdin = child.stdin.take().expect("stdin");
        let stdout = child.stdout.take().expect("stdout");

        let mut client = Self {
            child,
            stdin: std::io::BufWriter::new(stdin),
            reader: BufReader::new(stdout),
            next_id: 1,
            project_dir: project_dir.to_path_buf(),
        };

        let deadline = std::time::Instant::now() + Duration::from_secs(10);
        let mut last_err = String::new();
        while std::time::Instant::now() < deadline {
            match client.initialize() {
                Ok(_) => return client,
                Err(e) => {
                    last_err = e;
                    std::thread::sleep(Duration::from_millis(100));
                }
            }
        }
        panic!("MCP server did not become ready within 10s: {}", last_err);
    }

    pub fn send_request(
        &mut self,
        method: &str,
        params: serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        let id = self.next_id;
        self.next_id += 1;

        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": method,
            "params": params,
        });

        let line = serde_json::to_string(&request).map_err(|e| e.to_string())?;
        writeln!(self.stdin, "{}", line).map_err(|e| e.to_string())?;
        self.stdin.flush().map_err(|e| e.to_string())?;

        let deadline = std::time::Instant::now() + Duration::from_secs(30);
        let mut response_line = String::new();

        while std::time::Instant::now() < deadline {
            response_line.clear();
            if self
                .reader
                .read_line(&mut response_line)
                .map_err(|e| e.to_string())?
                == 0
            {
                return Err("EOF from MCP server".into());
            }
            let trimmed = response_line.trim();
            if trimmed.is_empty() {
                continue;
            }
            let resp: serde_json::Value =
                serde_json::from_str(trimmed).map_err(|e| format!("parse error: {}", e))?;
            if resp.get("id").and_then(|v| v.as_u64()) == Some(id) {
                if let Some(err) = resp.get("error") {
                    let msg = err
                        .get("message")
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown");
                    return Err(msg.to_string());
                }
                return Ok(resp);
            }
        }

        Err("timeout waiting for response".into())
    }

    pub fn initialize(&mut self) -> Result<serde_json::Value, String> {
        self.send_request(
            "initialize",
            serde_json::json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": { "name": "e2e-test", "version": "1.0.0" },
            }),
        )
    }

    pub fn call_tool(
        &mut self,
        name: &str,
        args: serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        let resp = self.send_request(
            "tools/call",
            serde_json::json!({
                "name": name,
                "arguments": args,
            }),
        )?;
        let result = resp.get("result").ok_or_else(|| {
            format!(
                "no result in response: {}",
                serde_json::to_string(&resp).unwrap_or_default()
            )
        })?;
        if let Some(true) = result.get("isError").and_then(|v| v.as_bool()) {
            let msg = result
                .get("content")
                .and_then(|c| c.as_array())
                .and_then(|a| a.first())
                .and_then(|c| c.get("text"))
                .and_then(|t| t.as_str())
                .unwrap_or("unknown error");
            return Err(msg.to_string());
        }
        if let Some(content) = result.get("content").and_then(|c| c.as_array()) {
            if let Some(first) = content.first() {
                if let Some(text) = first.get("text").and_then(|t| t.as_str()) {
                    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(text) {
                        return Ok(parsed);
                    }
                    return Ok(serde_json::json!({ "text": text }));
                }
            }
        }
        Ok(result.clone())
    }

    pub fn close(&mut self) {
        let _ = self.stdin.flush();
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
        // The proxy leaves the spawned daemon running (it persists for future
        // clients); reap it so tests don't leak processes.
        kill_recorded_daemon(&self.project_dir);
    }
}

impl Drop for MCPClient {
    fn drop(&mut self) {
        self.close();
    }
}
