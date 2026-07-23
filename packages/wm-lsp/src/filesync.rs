use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, Ordering};
use tokio::sync::Mutex;
use crate::{LspError, server::LspServer};

/// Tracks open file state per LSP session
pub struct FileSync {
    ref_counts: HashMap<String, u32>,      // uri → ref count
    server: Mutex<Option<Box<LspServer>>>, // the LSP server (optional, None when idle)
    version: AtomicU32,
}

impl FileSync {
    pub fn new() -> Self {
        Self { ref_counts: HashMap::new(), server: Mutex::new(None), version: AtomicU32::new(1) }
    }

    pub fn set_server(&mut self, server: LspServer) {
        let server = Box::new(server);
        *self.server.blocking_lock() = Some(server);
    }

    pub async fn open_file(&mut self, uri: &str, text: &str, lang_id: &str) -> Result<(), LspError> {
        let entry = self.ref_counts.entry(uri.to_string()).or_insert(0);
        if *entry == 0 {
            // First open — send didOpen
            if let Some(server) = self.server.lock().await.as_mut() {
                let mut client = server.client.lock().await;
                client.did_open(uri, text, lang_id).await?;
            }
        }
        *entry += 1;
        Ok(())
    }

    pub async fn close_file(&mut self, uri: &str) -> Result<(), LspError> {
        if let Some(entry) = self.ref_counts.get_mut(uri) {
            *entry = entry.saturating_sub(1);
            if *entry == 0 {
                self.ref_counts.remove(uri);
                // Send didClose after TTL (simplified: immediate close)
                if let Some(server) = self.server.lock().await.as_mut() {
                    let mut client = server.client.lock().await;
                    client.did_close(uri).await?;
                }
            }
        }
        Ok(())
    }

    pub async fn file_changed(&mut self, uri: &str, text: &str) -> Result<(), LspError> {
        if self.ref_counts.contains_key(uri) {
            let version = self.version.fetch_add(1, Ordering::SeqCst);
            if let Some(server) = self.server.lock().await.as_mut() {
                let mut client = server.client.lock().await;
                client.did_change(uri, text, version as i32).await?;
            }
        }
        Ok(())
    }
}
