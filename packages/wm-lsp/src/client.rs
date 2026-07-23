use lsp_types::*;
use crate::{LspError, transport::LspTransport};

pub struct LspClient {
    transport: LspTransport,
}

impl LspClient {
    pub fn new(transport: LspTransport) -> Self { Self { transport } }

    pub async fn initialize(&mut self, root_uri: &str, capabilities: ClientCapabilities) -> Result<InitializeResult, LspError> {
        let params = serde_json::json!({
            "processId": null,
            "rootUri": root_uri,
            "capabilities": capabilities,
        });
        let result = self.transport.send_request("initialize", params).await?;
        serde_json::from_value(result).map_err(|e| LspError::Protocol(e.to_string()))
    }

    pub async fn initialized(&mut self) -> Result<(), LspError> {
        self.transport.send_notification("initialized", serde_json::json!({})).await
    }

    pub async fn shutdown(&mut self) -> Result<(), LspError> {
        self.transport.send_request("shutdown", serde_json::json!(null)).await?;
        Ok(())
    }

    pub async fn exit(&mut self) -> Result<(), LspError> {
        self.transport.send_notification("exit", serde_json::json!({})).await
    }

    pub async fn did_open(&mut self, uri: &str, text: &str, lang_id: &str) -> Result<(), LspError> {
        self.transport.send_notification("textDocument/didOpen", serde_json::json!({
            "textDocument": { "uri": uri, "languageId": lang_id, "version": 1, "text": text }
        })).await
    }

    pub async fn did_close(&mut self, uri: &str) -> Result<(), LspError> {
        self.transport.send_notification("textDocument/didClose", serde_json::json!({
            "textDocument": { "uri": uri }
        })).await
    }

    pub async fn did_change(&mut self, uri: &str, text: &str, version: i32) -> Result<(), LspError> {
        self.transport.send_notification("textDocument/didChange", serde_json::json!({
            "textDocument": { "uri": uri, "version": version },
            "contentChanges": [{ "text": text }]
        })).await
    }

    pub async fn definition(&mut self, uri: &str, line: u32, col: u32) -> Result<GotoDefinitionResponse, LspError> {
        let result = self.transport.send_request("textDocument/definition", serde_json::json!({
            "textDocument": { "uri": uri },
            "position": { "line": line, "character": col }
        })).await?;
        serde_json::from_value(result).map_err(|e| LspError::Protocol(e.to_string()))
    }

    pub async fn references(&mut self, uri: &str, line: u32, col: u32, incl_decl: bool) -> Result<Vec<Location>, LspError> {
        let result = self.transport.send_request("textDocument/references", serde_json::json!({
            "textDocument": { "uri": uri },
            "position": { "line": line, "character": col },
            "context": { "includeDeclaration": incl_decl }
        })).await?;
        serde_json::from_value(result).map_err(|e| LspError::Protocol(e.to_string()))
    }

    pub async fn hover(&mut self, uri: &str, line: u32, col: u32) -> Result<Option<Hover>, LspError> {
        let result = self.transport.send_request("textDocument/hover", serde_json::json!({
            "textDocument": { "uri": uri },
            "position": { "line": line, "character": col }
        })).await?;
        serde_json::from_value(result).map_err(|e| LspError::Protocol(e.to_string()))
    }

    pub async fn goto_implementation(&mut self, uri: &str, line: u32, col: u32) -> Result<GotoDefinitionResponse, LspError> {
        let result = self.transport.send_request("textDocument/implementation", serde_json::json!({
            "textDocument": { "uri": uri },
            "position": { "line": line, "character": col }
        })).await?;
        serde_json::from_value(result).map_err(|e| LspError::Protocol(e.to_string()))
    }

    pub async fn workspace_symbol(&mut self, query: &str) -> Result<Vec<SymbolInformation>, LspError> {
        let result = self.transport.send_request("workspace/symbol", serde_json::json!({
            "query": query
        })).await?;
        serde_json::from_value(result).map_err(|e| LspError::Protocol(e.to_string()))
    }

    pub async fn rename(&mut self, uri: &str, line: u32, col: u32, new_name: &str) -> Result<WorkspaceEdit, LspError> {
        let result = self.transport.send_request("textDocument/rename", serde_json::json!({
            "textDocument": { "uri": uri },
            "position": { "line": line, "character": col },
            "newName": new_name
        })).await?;
        serde_json::from_value(result).map_err(|e| LspError::Protocol(e.to_string()))
    }
}
