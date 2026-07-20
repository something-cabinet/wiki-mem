//! pilot-runner: Tauri integration test runner using tauri-pilot CLI.
//!
//! Build: cargo build -p pilot-runner
//! Run:   pilot-runner --binary target/debug/wm-tauri.exe
//!
//! Launches the Tauri app, connects via tauri-pilot CLI, runs test groups,
//! reports pass/fail, and shuts down cleanly.

use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::thread::sleep;
use std::time::Duration;

mod tests;

// ─── CLI args ──────────────────────────────────────

struct Args {
    /// Path to the Tauri debug binary
    binary: PathBuf,
    /// Optional temp wiki directory for CRUD tests
    temp_wiki: Option<PathBuf>,
}

fn parse_args() -> Args {
    let mut args = std::env::args().skip(1);
    let mut binary = None;
    let mut temp_wiki = None;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--binary" => binary = args.next().map(PathBuf::from),
            "--temp-wiki" => temp_wiki = args.next().map(PathBuf::from),
            _ => {}
        }
    }

    Args {
        binary: binary.unwrap_or_else(|| {
            eprintln!("Usage: pilot-runner --binary <path> [--temp-wiki <path>]");
            std::process::exit(1);
        }),
        temp_wiki,
    }
}

// ─── App lifecycle ─────────────────────────────────

struct AppInstance {
    process: Option<Child>,
}

impl AppInstance {
    fn launch(binary: &PathBuf, wiki_dir: Option<&PathBuf>) -> Self {
        let mut cmd = Command::new(binary);
        cmd.stdout(Stdio::null())
            .stderr(Stdio::null());

        // If a temp wiki is provided, set the working directory so the app picks it up
        if let Some(wiki) = wiki_dir {
            cmd.current_dir(wiki.parent().unwrap_or(wiki));
        }

        println!("🔧 Launching Tauri app: {}", binary.display());
        let mut process = cmd.spawn().unwrap_or_else(|e| {
            eprintln!("❌ Failed to launch Tauri app: {}", e);
            std::process::exit(1);
        });

        // Wait for pilot socket to be ready
        println!("⏳ Waiting for pilot socket...");
        for _ in 0..30 {
            let output = Command::new("tauri-pilot")
                .arg("ping")
                .arg("--json")
                .output()
                .ok();
            if let Some(out) = output {
                if out.status.success() {
                    let stdout = String::from_utf8_lossy(&out.stdout);
                    if stdout.contains("\"ok\"") || stdout.contains("\"status\"") {
                        println!("✅ Pilot ready");
                        return Self {
                            process: Some(process),
                        };
                    }
                }
            }
            sleep(Duration::from_millis(500));
        }

        eprintln!("❌ Timed out waiting for pilot socket");
        let _ = process.kill();
        std::process::exit(1);
    }

    fn stop(&mut self) {
        if let Some(mut child) = self.process.take() {
            println!("🛑 Shutting down Tauri app...");
            let _ = child.kill();
            let _ = child.wait();
            println!("✅ App stopped");
        }
    }
}

impl Drop for AppInstance {
    fn drop(&mut self) {
        self.stop();
    }
}

// ─── Pilot IPC helper ─────────────────────────────

/// Call a Tauri command via tauri-pilot CLI and return the JSON result.
/// Args are passed as --args flag to support payload wrapping.
fn pilot_ipc_with_args(cmd: &str, args_json: &str) -> Result<serde_json::Value, String> {
    let output = Command::new("tauri-pilot")
        .arg("ipc")
        .arg(cmd)
        .arg("--args")
        .arg(args_json)
        .arg("--json")
        .output()
        .map_err(|e| format!("Failed to run tauri-pilot: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("tauri-pilot failed: {}", stderr.trim()));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    if stdout.trim().is_empty() {
        return Err("empty response from tauri-pilot".to_string());
    }

    serde_json::from_str(&stdout).map_err(|e| format!("JSON parse error: {} — raw: {}", e, stdout.trim()))
}

// ─── Test runner ──────────────────────────────────

struct TestResults {
    passed: Vec<String>,
    failed: Vec<(String, String)>,
}

impl TestResults {
    fn new() -> Self {
        Self {
            passed: Vec::new(),
            failed: Vec::new(),
        }
    }

    fn pass(&mut self, name: &str) {
        println!("  ✅ {}", name);
        self.passed.push(name.to_string());
    }

    fn fail(&mut self, name: &str, reason: &str) {
        println!("  ❌ {}: {}", name, reason);
        self.failed.push((name.to_string(), reason.to_string()));
    }

    fn report(&self) -> bool {
        println!("\n══════════════════════════");
        println!("  Passed: {}", self.passed.len());
        println!("  Failed: {}", self.failed.len());
        println!("══════════════════════════");
        for (name, reason) in &self.failed {
            println!("  ❌ {}: {}", name, reason);
        }
        self.failed.is_empty()
    }
}

// ─── Main ─────────────────────────────────────────

fn main() {
    let args = parse_args();

    let mut results = TestResults::new();

    {
        // Read-only group: runs against real project wiki
        let _app = AppInstance::launch(&args.binary, None);
        sleep(Duration::from_millis(1000)); // extra settling time

        tests::ipc::run_readonly_tests(&mut results);

        // App drops here → auto-shutdown
    }

    if let Some(ref temp_wiki) = args.temp_wiki {
        // CRUD group: runs against temp wiki
        let _app = AppInstance::launch(&args.binary, Some(temp_wiki));
        sleep(Duration::from_millis(1000));

        tests::ipc::run_crud_tests(&mut results);

        // App drops here → auto-shutdown
    }

    if results.report() {
        std::process::exit(0);
    } else {
        std::process::exit(1);
    }
}
