#[path = "helpers/setup.rs"]
mod setup;
use setup::setup_test_project;

use std::io::Read;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

const READY_DEADLINE_SECS: u64 = 30;

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
    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind ephemeral port");
    listener.local_addr().expect("local addr").port()
}

fn http_status(port: u16, path: &str) -> Option<u16> {
    use std::io::Write;
    use std::net::TcpStream;

    let addr = format!("127.0.0.1:{port}");
    let Ok(mut stream) = TcpStream::connect(&addr) else {
        return None;
    };
    let _ = stream.set_read_timeout(Some(Duration::from_secs(1)));
    let request =
        format!("GET {path} HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nConnection: close\r\n\r\n");
    if stream.write_all(request.as_bytes()).is_err() {
        return None;
    }
    let mut buf = [0u8; 4096];
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
    let mut tmp = [0u8; 4096];
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
                .args(["-9", &format!("-{}", self.child.id())])
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

fn wait_for_lifecycle_lines(proc: &WebProcess) -> String {
    let deadline = Instant::now() + Duration::from_secs(READY_DEADLINE_SECS);
    loop {
        let output = proc.output();
        let lines = [
            "Starting wm-server",
            "wm-server started",
            "Starting wm-web",
            "wm-web started",
        ];
        if lines.iter().all(|line| output.contains(line)) {
            return output;
        }
        assert!(
            Instant::now() < deadline,
            "lifecycle lines missing from output:\n{output}"
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

#[test]
fn wm_cli_web_lifecycle_logs_in_order() {
    let (_dir, root) = setup_test_project();
    let port = free_port();

    let mut proc = WebProcess::spawn(&root, port);
    wait_for_health(&mut proc, port);
    let output = wait_for_lifecycle_lines(&proc);
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
