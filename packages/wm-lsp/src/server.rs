use std::sync::Arc;
use tokio::sync::Mutex;
use crate::{LspError, transport::LspTransport, client::LspClient};

pub struct LspServer {
    pub language: String,
    pub client: tokio::sync::Mutex<LspClient>,
    pub readiness: tokio::sync::watch::Receiver<bool>,
    pub diagnostics_cache: Arc<dashmap::DashMap<String, Vec<lsp_types::Diagnostic>>>,
}

impl LspServer {
    pub async fn start(binary: &str, args: &[String], root_uri: &str, lang_id: &str) -> Result<Self, LspError> {
        let (transport, _child) = LspTransport::spawn(binary, args).await?;
        let mut client = LspClient::new(transport);

        let capabilities = lsp_types::ClientCapabilities::default();
        client.initialize(root_uri, capabilities).await?;
        client.initialized().await?;

        let (readiness_tx, readiness_rx) = tokio::sync::watch::channel(false);
        let diag_cache: Arc<dashmap::DashMap<String, Vec<lsp_types::Diagnostic>>> = Arc::new(dashmap::DashMap::new());

        // Simulate readiness (in production, subscribe to $/progress)
        let r_tx = readiness_tx.clone();
        tokio::spawn(async move {
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            let _ = r_tx.send(true);
        });

        Ok(Self {
            language: lang_id.to_string(),
            client: Mutex::new(client),
            readiness: readiness_rx,
            diagnostics_cache: diag_cache,
        })
    }

    pub async fn stop(&mut self) -> Result<(), LspError> {
        let mut client = self.client.lock().await;
        client.shutdown().await?;
        client.exit().await?;
        Ok(())
    }
}
