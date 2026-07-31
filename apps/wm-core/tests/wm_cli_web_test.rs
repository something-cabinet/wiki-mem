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
