use std::io::Read;
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

/// The wm-server binary next to the test binary (same target dir as wm-cli).
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

/// Point the project at a dedicated free port before running the command so
/// migrated CLI commands (daemon-discovery via `.wm/server.json`) don't race on
/// the default 4090.
fn setup_daemon_port(dir: &std::path::Path) {
    let port = free_port();
    let server_json = dir.join(".wm").join("server.json");
    let _ = std::fs::create_dir_all(server_json.parent().unwrap());
    let _ = std::fs::write(
        &server_json,
        serde_json::to_string(&serde_json::json!({
            "port": port,
            "pid": 0,
            "started_at": "test",
        }))
        .unwrap(),
    );
}

/// Kill the daemon recorded in `.wm/server.json` (spawned by a migrated CLI
/// command) so tests don't leak wm-server processes on ephemeral ports.
fn kill_recorded_daemon(dir: &std::path::Path) {
    let path = dir.join(".wm").join("server.json");
    let Ok(content) = std::fs::read_to_string(&path) else {
        return;
    };
    let Ok(value) = serde_json::from_str::<serde_json::Value>(&content) else {
        return;
    };
    if let Some(pid) = value.get("pid").and_then(serde_json::Value::as_u64) {
        if pid == 0 {
            return;
        }
        #[cfg(windows)]
        let _ = std::process::Command::new("taskkill")
            .args(["/F", "/T", "/PID", &pid.to_string()])
            .status();
        #[cfg(not(windows))]
        let _ = std::process::Command::new("kill")
            .arg("-9")
            .arg(pid.to_string())
            .status();
    }
}

#[derive(Debug)]
pub struct CliResult {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
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

pub fn run_cli(dir: &std::path::Path, args: &[&str]) -> CliResult {
    setup_daemon_port(dir);
    let bin = get_binary_path();
    let mut cmd = Command::new(&bin);
    cmd.args(args);
    cmd.current_dir(dir);
    cmd.env("NO_COLOR", "1");
    cmd.env("WM_SERVER_PATH", get_server_binary_path());
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

    let timeout = Duration::from_secs(120);
    let start = std::time::Instant::now();
    let res = loop {
        if start.elapsed() >= timeout {
            let _ = kill_process(&mut child);
            break CliResult {
                stdout: String::new(),
                stderr: "Timeout after 120s".to_string(),
                exit_code: -1,
            };
        }
        match child.try_wait() {
            Ok(Some(status)) => {
                let mut stdout = Vec::new();
                let mut stderr = Vec::new();
                if let Some(ref mut out) = child.stdout {
                    let _ = out.read_to_end(&mut stdout);
                }
                if let Some(ref mut err) = child.stderr {
                    let _ = err.read_to_end(&mut stderr);
                }
                break CliResult {
                    stdout: String::from_utf8_lossy(&stdout).to_string(),
                    stderr: String::from_utf8_lossy(&stderr).to_string(),
                    exit_code: status.code().unwrap_or(-1),
                };
            }
            Ok(None) => {}
            Err(e) => {
                let _ = kill_process(&mut child);
                break CliResult {
                    stdout: String::new(),
                    stderr: format!("Wait error: {}", e),
                    exit_code: -1,
                };
            }
        }
        std::thread::sleep(Duration::from_millis(50));
    };
    kill_recorded_daemon(dir);
    res
}
