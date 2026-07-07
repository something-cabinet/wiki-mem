// ─── E2E Test Helpers ─────────────────────────────────────────
// Following Knowns pattern from tests/helpers_test.go:
//   setup_test_project(), get_binary_path(), run_cli(), MCPClient

use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::time::Duration;

/// Find the wm-cli binary path.
/// Respects TEST_BINARY env var; defaults to workspace target/debug/wm-cli(.exe).
pub fn get_binary_path() -> PathBuf {
    if let Ok(p) = std::env::var("TEST_BINARY") {
        return PathBuf::from(p);
    }
    // Derive from test binary path: .../target/debug/deps/wm_core-<hash>.exe
    // -> .../target/debug/wm-cli.exe
    let exe = std::env::current_exe().expect("current exe");
    let mut path = exe.parent().unwrap(); // deps/
    // Go up to debug/
    if path.ends_with("deps") {
        path = path.parent().unwrap();
    }
    let bin_name = if cfg!(windows) { "wm-cli.exe" } else { "wm-cli" };
    path.join(bin_name)
}

/// CLIResult holds output from a CLI command.
#[derive(Debug)]
pub struct CliResult {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
}

/// Run a CLI command with a 60s timeout.
pub fn run_cli(dir: &std::path::Path, args: &[&str]) -> CliResult {
    let bin = get_binary_path();
    let mut cmd = Command::new(&bin);
    cmd.args(args);
    cmd.current_dir(dir);
    cmd.env("NO_COLOR", "1");
    // Unset WM_PROJECT to prevent the CLI from using the host project instead of test project
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

/// Cross-platform child process kill.
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

/// Run a CLI command with a custom timeout.
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
        // Check if process exited
        match child.try_wait() {
            Ok(Some(status)) => {
                // Process exited — get full output
                // We waited, so we need to collect remaining stdout/stderr
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
            Ok(None) => {} // still running
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

/// Create a temporary test project with .wm/config.json.
/// Returns the project directory path.
pub fn setup_test_project() -> (tempfile::TempDir, PathBuf) {
    let dir = tempfile::tempdir().expect("temp dir");
    let root = dir.path().to_path_buf();

    // Create .wm directory structure
    let wm_dir = root.join(".wm");
    std::fs::create_dir_all(wm_dir.join("wiki")).expect("create .wm/wiki");
    std::fs::create_dir_all(wm_dir.join("sources")).expect("create .wm/sources");
    std::fs::create_dir_all(wm_dir.join("state")).expect("create .wm/state");
    // Create .agents/skills/ for skill files
    std::fs::create_dir_all(root.join(".agents").join("skills")).expect("create .agents/skills");

    // Create .wm/memory/ for memory entries
    std::fs::create_dir_all(wm_dir.join("memory")).expect("create .wm/memory");

    // Create wiki subdirs
    for sub in &[
        "tasks", "specs", "concepts", "patterns", "decisions", "howto", "reference",
    ] {
        std::fs::create_dir_all(wm_dir.join("wiki").join(sub)).expect("create wiki subdir");
    }

    // Write config.json
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

    // Create AGENTS.md
    let agents = "# AGENTS.md — Wiki Memory Engine Agent Handbook\n\n## Wiki Conventions\n...\n";
    std::fs::write(wm_dir.join("AGENTS.md"), agents).expect("write AGENTS.md");

    (dir, root)
}

// ─── MCP Client ──────────────────────────────────────────────

/// MCPClient manages a child MCP server process via stdio JSON-RPC.
pub struct MCPClient {
    child: Child,
    stdin: std::io::BufWriter<std::process::ChildStdin>,
    reader: BufReader<std::process::ChildStdout>,
    next_id: u64,
}

impl MCPClient {
    /// Spawn `wm serve` and return a connected client.
    pub fn start(project_dir: &std::path::Path) -> Self {
        let bin = get_binary_path();
        let mut cmd = Command::new(&bin);
        cmd.arg("serve");
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

        // Active readiness: retry initialize with backoff until server responds
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

    /// Send a JSON-RPC request and read the response line.
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

        // Read until we get our response (skip notifications/other responses)
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
                // Check for error
                if let Some(err) = resp.get("error") {
                    let msg = err.get("message").and_then(|v| v.as_str()).unwrap_or("unknown");
                    return Err(msg.to_string());
                }
                return Ok(resp);
            }
        }

        Err("timeout waiting for response".into())
    }

    /// Send initialize handshake.
    pub fn initialize(&mut self) -> Result<serde_json::Value, String> {
        self.send_request("initialize", serde_json::json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": { "name": "e2e-test", "version": "1.0.0" },
        }))
    }

    /// Call an MCP tool and return the result content.
    /// WM Engine returns tool results directly in the `result` field (not wrapped in MCP content[]).
    pub fn call_tool(&mut self, name: &str, args: serde_json::Value) -> Result<serde_json::Value, String> {
        let resp = self.send_request("tools/call", serde_json::json!({
            "name": name,
            "arguments": args,
        }))?;

        // WM Engine returns result directly (not wrapped in MCP content[])
        let result = resp.get("result").ok_or_else(|| {
            format!("no result in response: {}", serde_json::to_string(&resp).unwrap_or_default())
        })?;
        Ok(result.clone())
    }

    /// List available tools.
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

    /// Close the MCP server.
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

// ─── Assertion Helpers ───────────────────────────────────────

/// Assert that a CLI command succeeded (exit code 0).
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

/// Assert that output contains a substring.
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
