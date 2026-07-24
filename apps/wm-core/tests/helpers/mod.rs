
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::time::Duration;

pub fn get_binary_path() -> PathBuf {
    if let Ok(p) = std::env::var("TEST_BINARY") {
        return PathBuf::from(p);
    }
    let exe = std::env::current_exe().expect("current exe");
    let mut path = exe.parent().unwrap();
    if path.ends_with("deps") {
        path = path.parent().unwrap();
    }
    let bin_name = if cfg!(windows) { "wm-cli.exe" } else { "wm-cli" };
    path.join(bin_name)
}

#[derive(Debug)]
pub struct CliResult {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
}

pub fn run_cli_with_stdin(dir: &std::path::Path, args: &[&str], stdin_input: &str) -> CliResult {
    let bin = get_binary_path();
    let mut cmd = Command::new(&bin);
    cmd.args(args);
    cmd.current_dir(dir);
    cmd.env("NO_COLOR", "1");
    cmd.env_remove("WM_PROJECT");
    cmd.stdin(Stdio::piped());
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    let mut child = match cmd.spawn() {
        Ok(c) => c,
        Err(e) => {
            return CliResult {
                stdout: String::new(),
                stderr: format!("Failed to spawn: {}", e),
                exit_code: -1,
            };
        }
    };

    if let Some(stdin) = child.stdin.take() {
        let mut writer = std::io::BufWriter::new(stdin);
        let _ = writer.write_all(stdin_input.as_bytes());
        let _ = writer.flush();
    }

    let timeout = Duration::from_secs(60);
    let start = std::time::Instant::now();
    loop {
        if start.elapsed() >= timeout {
            let _ = kill_process(&mut child);
            return CliResult {
                stdout: String::new(),
                stderr: "Timeout after 60s".to_string(),
                exit_code: -1,
            };
        }
        match child.try_wait() {
            Ok(Some(status)) => {
                let mut stdout = Vec::new();
                let mut stderr = Vec::new();
                use std::io::Read;
                if let Some(ref mut out) = child.stdout {
                    let _ = out.read_to_end(&mut stdout);
                }
                if let Some(ref mut err) = child.stderr {
                    let _ = err.read_to_end(&mut stderr);
                }
                return CliResult {
                    stdout: String::from_utf8_lossy(&stdout).to_string(),
                    stderr: String::from_utf8_lossy(&stderr).to_string(),
                    exit_code: status.code().unwrap_or(-1),
                };
            }
            Ok(None) => {}
            Err(e) => {
                let _ = kill_process(&mut child);
                return CliResult {
                    stdout: String::new(),
                    stderr: format!("Wait error: {}", e),
                    exit_code: -1,
                };
            }
        }
        std::thread::sleep(Duration::from_millis(50));
    }
}

pub fn run_cli(dir: &std::path::Path, args: &[&str]) -> CliResult {
    let bin = get_binary_path();
    let mut cmd = Command::new(&bin);
    cmd.args(args);
    cmd.current_dir(dir);
    cmd.env("NO_COLOR", "1");
    cmd.env_remove("WM_PROJECT");
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    let mut child = match cmd.spawn() {
        Ok(c) => c,
        Err(e) => {
            return CliResult {
                stdout: String::new(),
                stderr: format!("Failed to spawn: {}", e),
                exit_code: -1,
            };
        }
    };

    let timeout = Duration::from_secs(60);
    let start = std::time::Instant::now();
    loop {
        if start.elapsed() >= timeout {
            let _ = kill_process(&mut child);
            return CliResult {
                stdout: String::new(),
                stderr: "Timeout after 60s".to_string(),
                exit_code: -1,
            };
        }
        match child.try_wait() {
            Ok(Some(status)) => {
                let mut stdout = Vec::new();
                let mut stderr = Vec::new();
                use std::io::Read;
                if let Some(ref mut out) = child.stdout {
                    let _ = out.read_to_end(&mut stdout);
                }
                if let Some(ref mut err) = child.stderr {
                    let _ = err.read_to_end(&mut stderr);
                }
                return CliResult {
                    stdout: String::from_utf8_lossy(&stdout).to_string(),
                    stderr: String::from_utf8_lossy(&stderr).to_string(),
                    exit_code: status.code().unwrap_or(-1),
                };
            }
            Ok(None) => {}
            Err(e) => {
                let _ = kill_process(&mut child);
                return CliResult {
                    stdout: String::new(),
                    stderr: format!("Wait error: {}", e),
                    exit_code: -1,
                };
            }
        }
        std::thread::sleep(Duration::from_millis(50));
    }
}

fn kill_process(child: &mut Child) {
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
}

pub fn run_cli_with_timeout(
    dir: &std::path::Path,
    args: &[&str],
    timeout: Duration,
) -> CliResult {
    let bin = get_binary_path();
    let mut cmd = Command::new(&bin);
    cmd.args(args);
    cmd.current_dir(dir);
    cmd.env("NO_COLOR", "1");
    cmd.env_remove("WM_PROJECT");

    let mut child = match cmd.spawn() {
        Ok(c) => c,
        Err(e) => {
            return CliResult {
                stdout: String::new(),
                stderr: format!("Failed to spawn: {}", e),
                exit_code: -1,
            };
        }
    };

    let start = std::time::Instant::now();
    loop {
        if start.elapsed() >= timeout {
            let _ = kill_process(&mut child);
            return CliResult {
                stdout: String::new(),
                stderr: "Timeout".to_string(),
                exit_code: -1,
            };
        }
        match child.try_wait() {
            Ok(Some(status)) => {
                let mut stdout = Vec::new();
                let mut stderr = Vec::new();
                use std::io::Read;
                if let Some(ref mut out) = child.stdout {
                    let _ = out.read_to_end(&mut stdout);
                }
                if let Some(ref mut err) = child.stderr {
                    let _ = err.read_to_end(&mut stderr);
                }
                return CliResult {
                    stdout: String::from_utf8_lossy(&stdout).to_string(),
                    stderr: String::from_utf8_lossy(&stderr).to_string(),
                    exit_code: status.code().unwrap_or(-1),
                };
            }
            Ok(None) => {}
            Err(e) => {
                return CliResult {
                    stdout: String::new(),
                    stderr: format!("Wait error: {}", e),
                    exit_code: -1,
                };
            }
        }
        std::thread::sleep(Duration::from_millis(50));
    }
}

pub fn setup_test_project() -> (tempfile::TempDir, PathBuf) {
    let dir = tempfile::tempdir().expect("temp dir");
    let root = dir.path().to_path_buf();

    let wm_dir = root.join(".wm");
    std::fs::create_dir_all(wm_dir.join("wiki")).expect("create .wm/wiki");
    std::fs::create_dir_all(wm_dir.join("sources")).expect("create .wm/sources");
    std::fs::create_dir_all(wm_dir.join("state")).expect("create .wm/state");
    std::fs::create_dir_all(root.join(".agents").join("skills")).expect("create .agents/skills");

    std::fs::create_dir_all(wm_dir.join("memory")).expect("create .wm/memory");

    for sub in &[
        "tasks", "specs", "concepts", "patterns", "decisions", "howto", "reference",
    ] {
        std::fs::create_dir_all(wm_dir.join("wiki").join(sub)).expect("create wiki subdir");
    }

    let config = serde_json::json!({
        "project_name": "",
        "schema_version": 1,
        "embedding": {
            "model_name": "bge-small-en-v1.5",
            "dimensions": 384,
            "batch_size": 32
        },
        "permissions": { "preset": "read-write" },
        "custom_edge_types": [],
        "source_dirs": ["docs/", "specs/"],
        "source_extensions": ["md", "yaml", "txt"],
        "search": {
            "default_mode": "hybrid",
            "default_limit": 20,
            "rrf_k": 60
        }
    });
    std::fs::write(
        wm_dir.join("config.json"),
        serde_json::to_string_pretty(&config).unwrap(),
    )
    .expect("write config.json");

    let agents = "# AGENTS.md — Wiki Memory Engine Agent Handbook\n\n## Wiki Conventions\n...\n";
    std::fs::write(wm_dir.join("AGENTS.md"), agents).expect("write AGENTS.md");

    (dir, root)
}


pub struct MCPClient {
    child: Child,
    stdin: std::io::BufWriter<std::process::ChildStdin>,
    reader: BufReader<std::process::ChildStdout>,
    next_id: u64,
}

impl MCPClient {
    pub fn start(project_dir: &std::path::Path) -> Self {
        let bin = get_binary_path();
        let mut cmd = Command::new(&bin);
        cmd.arg("mcp");
        cmd.current_dir(project_dir);
        cmd.env("NO_COLOR", "1");
        cmd.env_remove("WM_PROJECT");
        cmd.stdin(Stdio::piped());
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::null());

        let mut child = cmd.spawn().expect("failed to spawn wm serve");
        let stdin = child.stdin.take().expect("stdin");
        let stdout = child.stdout.take().expect("stdout");

        let mut client = Self {
            child,
            stdin: std::io::BufWriter::new(stdin),
            reader: BufReader::new(stdout),
            next_id: 1,
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

    pub fn send_request(&mut self, method: &str, params: serde_json::Value) -> Result<serde_json::Value, String> {
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
            if self.reader.read_line(&mut response_line).map_err(|e| e.to_string())? == 0 {
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
                    let msg = err.get("message").and_then(|v| v.as_str()).unwrap_or("unknown");
                    return Err(msg.to_string());
                }
                return Ok(resp);
            }
        }

        Err("timeout waiting for response".into())
    }

    pub fn initialize(&mut self) -> Result<serde_json::Value, String> {
        self.send_request("initialize", serde_json::json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": { "name": "e2e-test", "version": "1.0.0" },
        }))
    }

    pub fn call_tool(&mut self, name: &str, args: serde_json::Value) -> Result<serde_json::Value, String> {
        let resp = self.send_request("tools/call", serde_json::json!({
            "name": name,
            "arguments": args,
        }))?;
        let result = resp.get("result").ok_or_else(|| {
            format!("no result in response: {}",
                serde_json::to_string(&resp).unwrap_or_default())
        })?;
        if let Some(true) = result.get("isError").and_then(|v| v.as_bool()) {
            let msg = result.get("content")
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

    pub fn list_tools(&mut self) -> Result<Vec<String>, String> {
        let resp = self.send_request("tools/list", serde_json::json!({}))?;
        let result = resp.get("result").ok_or("no result")?;
        let tools = result
            .get("tools")
            .and_then(|t| t.as_array())
            .ok_or("no tools array in result")?;
        Ok(tools
            .iter()
            .filter_map(|t| t.get("name").and_then(|n| n.as_str().map(String::from)))
            .collect())
    }

    pub fn close(&mut self) {
        let _ = self.stdin.flush();
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

impl Drop for MCPClient {
    fn drop(&mut self) {
        self.close();
    }
}


#[macro_export]
macro_rules! assert_success {
    ($res:expr) => {
        assert!(
            $res.exit_code == 0,
            "expected exit code 0, got {}:\nstdout: {}\nstderr: {}",
            $res.exit_code,
            $res.stdout,
            $res.stderr
        );
    };
}

#[macro_export]
macro_rules! assert_contains {
    ($haystack:expr, $needle:expr) => {{
        let haystack = &$haystack;
        let needle = &$needle;
        assert!(
            haystack.contains(needle),
            "expected output to contain {:?}\ngot: {:?}",
            needle,
            haystack
        );
    }};
}
 