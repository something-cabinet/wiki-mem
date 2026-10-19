use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, SystemTime};

const BINARY_OVERRIDE_ENV: &str = "TEST_BINARY";
const CLI_SOURCE_SUBDIR: &str = "../wm-cli/src";
const STALE_BINARY_HINT: &str =
    "stale wm-cli binary: run `cargo build -p wm-cli` before the CLI tests";

fn newest_source_mtime() -> SystemTime {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join(CLI_SOURCE_SUBDIR);
    walkdir::WalkDir::new(root)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file())
        .filter_map(|entry| entry.metadata().ok())
        .filter_map(|meta| meta.modified().ok())
        .max()
        .unwrap_or(SystemTime::UNIX_EPOCH)
}

fn assert_binary_fresh(binary: &Path) {
    if std::env::var(BINARY_OVERRIDE_ENV).is_ok() {
        return;
    }
    let Ok(meta) = std::fs::metadata(binary) else {
        return;
    };
    let Ok(binary_mtime) = meta.modified() else {
        return;
    };
    if binary_mtime < newest_source_mtime() {
        panic!("{STALE_BINARY_HINT}");
    }
}

fn get_binary_path() -> PathBuf {
    if let Ok(p) = std::env::var(BINARY_OVERRIDE_ENV) {
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
    let binary = path.join(bin_name);
    assert_binary_fresh(&binary);
    binary
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

fn spawn_cli(dir: &std::path::Path, args: &[&str], pipe_stdin: bool) -> Child {
    let bin = get_binary_path();
    let mut cmd = Command::new(&bin);
    cmd.args(args);
    cmd.current_dir(dir);
    cmd.env("NO_COLOR", "1");
    cmd.env("HOME", dir);
    cmd.env("USERPROFILE", dir);
    cmd.env_remove("WM_PROJECT");
    if pipe_stdin {
        cmd.stdin(Stdio::piped());
    }
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());
    cmd.spawn().expect("spawn wm-cli")
}

fn wait_for_exit(child: &mut Child) -> CliResult {
    let timeout = Duration::from_secs(120);
    let start = std::time::Instant::now();
    loop {
        if start.elapsed() >= timeout {
            kill_process(child);
            return CliResult {
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
                return CliResult {
                    stdout: String::from_utf8_lossy(&stdout).to_string(),
                    stderr: String::from_utf8_lossy(&stderr).to_string(),
                    exit_code: status.code().unwrap_or(-1),
                };
            }
            Ok(None) => {}
            Err(e) => {
                kill_process(child);
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

pub fn run_cli_with_stdin(dir: &std::path::Path, args: &[&str], stdin_input: &str) -> CliResult {
    let mut child = spawn_cli(dir, args, true);
    if let Some(stdin) = child.stdin.take() {
        let mut writer = std::io::BufWriter::new(stdin);
        let _ = writer.write_all(stdin_input.as_bytes());
        let _ = writer.flush();
    }
    wait_for_exit(&mut child)
}

pub fn run_cli(dir: &std::path::Path, args: &[&str]) -> CliResult {
    let mut child = spawn_cli(dir, args, false);
    wait_for_exit(&mut child)
}
