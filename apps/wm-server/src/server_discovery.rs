//! Server discovery via `.wm/server.json`.
//!
//! The `wm-server` daemon records its bound port (plus pid and start time) in
//! a small JSON file under the project's `.wm/` directory. CLI and MCP clients
//! read this file and health-check the recorded port to decide whether to
//! connect to the running daemon or spawn a fresh one — avoiding duplicate
//! `EngineState` instances.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Duration;

use serde::{Deserialize, Serialize};
use wm_constants::*;

/// Name of the discovery file, relative to the project's `.wm/` directory.
pub const SERVER_JSON_FILE: &str = "server.json";

const HEALTH_PATH: &str = "/api/health";

/// On-disk record describing a running `wm-server` instance.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ServerInfo {
    pub port: u16,
    pub pid: u32,
    pub started_at: String,
}

/// Absolute path of the discovery file for a given project root.
pub fn server_json_path(root: &Path) -> PathBuf {
    root.join(WM_DIR).join(SERVER_JSON_FILE)
}

/// Atomically write the discovery file describing the current daemon instance.
///
/// The file is written to a sibling temp file first and renamed into place so
/// readers never observe a partially-written payload.
pub fn write_server_json(root: &Path, port: u16) -> anyhow::Result<PathBuf> {
    let info = ServerInfo {
        port,
        pid: std::process::id(),
        started_at: chrono::Utc::now().to_rfc3339(),
    };

    let path = server_json_path(root);
    let parent = path
        .parent()
        .ok_or_else(|| anyhow::anyhow!("server.json path has no parent directory"))?;
    std::fs::create_dir_all(parent)?;

    let tmp = parent.join(format!("{SERVER_JSON_FILE}.tmp"));
    {
        let mut file = std::fs::File::create(&tmp)?;
        serde_json::to_writer(&mut file, &info)?;
        file.write_all(b"\n")?;
        file.sync_all()?;
    }
    std::fs::rename(&tmp, &path)?;

    Ok(path)
}

/// Read the full discovery record, if a well-formed file exists.
pub fn read_server_info(root: &Path) -> Option<ServerInfo> {
    let content = std::fs::read_to_string(server_json_path(root)).ok()?;
    serde_json::from_str(&content).ok()
}

/// Read the port recorded in `.wm/server.json`, if any.
pub fn read_server_json(root: &Path) -> Option<u16> {
    read_server_info(root).map(|info| info.port)
}

/// Whether a daemon recorded for `root` on `port` is actually reachable.
///
/// Returns `true` only when the discovery file names the expected port *and*
/// `GET /api/health` responds within a short timeout. This is the check CLI/MCP
/// uses to decide whether to connect or spawn a fresh daemon.
pub fn is_running(root: &Path, port: u16) -> bool {
    let recorded = match read_server_json(root) {
        Some(p) if p == port => p,
        _ => return false,
    };
    http_status(recorded, HEALTH_PATH).is_some()
}

/// Best-effort HTTP status code for `GET {path}` on `127.0.0.1:{port}`.
fn http_status(port: u16, path: &str) -> Option<u16> {
    use std::io::{Read, Write};

    let addr = format!("{LOCALHOST_ADDR}:{port}");
    let mut stream = std::net::TcpStream::connect(&addr).ok()?;
    let _ = stream.set_read_timeout(Some(Duration::from_secs(HTTP_PROBE_READ_TIMEOUT_SECS)));
    let request = format!(
        "GET {path} HTTP/1.1\r\nHost: {LOCALHOST_ADDR}:{port}\r\nConnection: close\r\n\r\n"
    );
    stream.write_all(request.as_bytes()).ok()?;

    let mut buf = [0u8; HTTP_PROBE_BUF_LEN];
    let mut filled = 0;
    while filled < buf.len() {
        let n = match stream.read(&mut buf[filled..]) {
            Ok(0) => break,
            Ok(n) => n,
            Err(_) => break,
        };
        filled += n;
        if buf[..filled].contains(&b'\n') {
            break;
        }
    }

    let head = String::from_utf8_lossy(&buf[..filled]);
    let mut parts = head.lines().next()?.split_whitespace();
    parts.nth(1)?.parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_root(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "wm-server-discovery-{tag}-{pid}",
            pid = std::process::id()
        ));
        std::fs::create_dir_all(dir.join(WM_DIR)).unwrap();
        dir
    }

    #[test]
    fn write_and_read_roundtrip() {
        let root = temp_root("roundtrip");
        let path = write_server_json(&root, 4090).unwrap();
        assert_eq!(path, server_json_path(&root));
        assert!(path.exists());

        assert_eq!(read_server_json(&root), Some(4090));
        let info = read_server_info(&root).unwrap();
        assert_eq!(info.port, 4090);
        assert_eq!(info.pid, std::process::id());
        assert!(!info.started_at.is_empty());

        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn read_missing_or_malformed_returns_none() {
        let root = temp_root("missing");
        assert_eq!(read_server_json(&root), None);

        std::fs::write(server_json_path(&root), "not json").unwrap();
        assert_eq!(read_server_json(&root), None);

        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn write_overwrites_previous_record() {
        let root = temp_root("overwrite");
        write_server_json(&root, 4090).unwrap();
        write_server_json(&root, 5000).unwrap();
        assert_eq!(read_server_json(&root), Some(5000));

        std::fs::remove_dir_all(&root).ok();
    }
}
