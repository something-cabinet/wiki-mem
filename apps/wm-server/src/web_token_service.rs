use std::io::Write;
use std::path::{Path, PathBuf};

use wm_constants::*;

const WEB_TOKEN_FILE: &str = "web-token";
const MCP_TOKEN_FILE: &str = "mcp-token";
const TOKEN_BYTES: usize = 32;
const TOKEN_HEADER: &str = "x-wm-token";

/// Which credential channel a token belongs to. Both use the same header
/// (`x-wm-token`) but distinct values: the MCP token is the privileged
/// credential for `/api/mcp/*`, the web token guards the read-only web API.
/// Neither token authorizes the other channel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    /// Read-only web API surface (`/api/*`, except `/api/mcp/*`).
    Web,
    /// Privileged MCP proxy channel (`/api/mcp/*`).
    Mcp,
}

impl TokenKind {
    pub fn file_name(self) -> &'static str {
        match self {
            TokenKind::Web => WEB_TOKEN_FILE,
            TokenKind::Mcp => MCP_TOKEN_FILE,
        }
    }
}

pub fn header_name() -> &'static str {
    TOKEN_HEADER
}

pub fn token_path(project_root: &Path, kind: TokenKind) -> PathBuf {
    project_root
        .join(WM_DIR)
        .join(STATE_DIR)
        .join(kind.file_name())
}

fn encode_hex(bytes: &[u8]) -> String {
    bytes.iter().fold(String::new(), |mut acc, b| {
        acc.push_str(&format!("{:02x}", b));
        acc
    })
}

pub fn generate_and_persist(project_root: &Path, kind: TokenKind) -> anyhow::Result<String> {
    let mut buf = [0u8; TOKEN_BYTES];
    getrandom::getrandom(&mut buf).map_err(|e| {
        anyhow::anyhow!(
            "failed to gather entropy for {} token: {e}",
            kind.file_name()
        )
    })?;
    let token = encode_hex(&buf);

    let path = token_path(project_root, kind);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let mut file = std::fs::File::create(&path)?;
    file.write_all(token.as_bytes())?;
    file.flush()?;
    restrict_permissions(&path)?;

    tracing::info!("{} token written to {}", kind.file_name(), path.display());
    Ok(token)
}

#[cfg(unix)]
fn restrict_permissions(path: &Path) -> anyhow::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    const MODE: u32 = 0o600;
    let mut perms = std::fs::metadata(path)?.permissions();
    perms.set_mode(MODE);
    std::fs::set_permissions(path, perms)?;
    Ok(())
}

#[cfg(not(unix))]
fn restrict_permissions(_path: &Path) -> anyhow::Result<()> {
    Ok(())
}
