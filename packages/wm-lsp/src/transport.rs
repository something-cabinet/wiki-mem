use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use dashmap::DashMap;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, ChildStdout};
use tokio::sync::oneshot;

use crate::LspError;

type ResponseSender = oneshot::Sender<Result<serde_json::Value, String>>;

pub struct LspTransport {
    stdin: ChildStdin,
    #[allow(dead_code)]
    reader_task: tokio::task::JoinHandle<()>,
    pending: Arc<DashMap<u64, ResponseSender>>,
    next_id: AtomicU64,
}

impl LspTransport {
    /// Spawn a child process and create transport connected to its stdio
    pub async fn spawn(command: &str, args: &[String]) -> Result<(Self, Child), LspError> {
        let mut child = tokio::process::Command::new(command)
            .args(args)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()?;

        let stdin = child.stdin.take().unwrap();
        let stdout = child.stdout.take().unwrap();

        let transport = Self::new(stdin, stdout);
        Ok((transport, child))
    }

    pub fn new(stdin: ChildStdin, stdout: ChildStdout) -> Self {
        let pending: Arc<DashMap<u64, ResponseSender>> = Arc::new(DashMap::new());
        let pending_clone = pending.clone();

        let reader_task = tokio::spawn(async move {
            let mut reader = BufReader::new(stdout);
            let mut buf = Vec::new();
            loop {
                buf.clear();
                // Read Content-Length header
                let mut header = String::new();
                match reader.read_line(&mut header).await {
                    Ok(0) | Err(_) => break,
                    Ok(_) => {}
                }
                let len: usize = header
                    .strip_prefix("Content-Length: ")
                    .and_then(|s| s.trim().parse().ok())
                    .unwrap_or(0);

                // Read empty line
                let mut empty = String::new();
                reader.read_line(&mut empty).await.ok();

                // Read body
                buf.resize(len, 0);
                reader.read_exact(&mut buf).await.ok();

                if let Ok(msg) = serde_json::from_slice::<serde_json::Value>(&buf) {
                    if let Some(id) = msg.get("id").and_then(|v| v.as_u64()) {
                        if let Some((_, sender)) = pending_clone.remove(&id) {
                            let result = msg.get("result").cloned().ok_or_else(|| {
                                msg.get("error")
                                    .and_then(|e| e.get("message").and_then(|m| m.as_str().map(String::from)))
                                    .unwrap_or_default()
                            });
                            let _ = sender.send(result);
                        }
                    }
                    // Handle notifications (publishDiagnostics, $/progress, etc.)
                }
            }
        });

        Self { stdin, reader_task, pending, next_id: AtomicU64::new(1) }
    }

    pub async fn send_request(&mut self, method: &str, params: serde_json::Value) -> Result<serde_json::Value, LspError> {
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        let (tx, rx) = oneshot::channel();
        self.pending.insert(id, tx);

        let body = serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": method,
            "params": params,
        });
        let body_str = serde_json::to_string(&body).map_err(|e| LspError::Protocol(e.to_string()))?;

        let header = format!("Content-Length: {}\r\n\r\n", body_str.len());
        self.stdin.write_all(header.as_bytes()).await?;
        self.stdin.write_all(body_str.as_bytes()).await?;
        self.stdin.flush().await?;

        tokio::time::timeout(std::time::Duration::from_secs(30), rx)
            .await
            .map_err(|_| LspError::Timeout { operation: method.to_string() })?
            .map_err(|_| LspError::Transport("channel closed".to_string()))?
            .map_err(|e| LspError::Protocol(e))
    }

    pub async fn send_notification(&mut self, method: &str, params: serde_json::Value) -> Result<(), LspError> {
        let body = serde_json::json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
        });
        let body_str = serde_json::to_string(&body).map_err(|e| LspError::Protocol(e.to_string()))?;
        let header = format!("Content-Length: {}\r\n\r\n", body_str.len());
        self.stdin.write_all(header.as_bytes()).await?;
        self.stdin.write_all(body_str.as_bytes()).await?;
        self.stdin.flush().await?;
        Ok(())
    }
}
