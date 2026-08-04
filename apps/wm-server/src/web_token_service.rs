use std::io::Write;
use std::path::{Path, PathBuf};

use wm_constants::*;

const TOKEN_FILE: &str = "web-token";
const TOKEN_BYTES: usize = 32;
const TOKEN_HEADER: &str = "x-wm-token";
const TOKEN_MODE: u32 = 0o600;

pub fn header_name() -> &'static str {
    TOKEN_HEADER
}

fn token_path(project_root: &Path) -> PathBuf {
    project_root.join(WM_DIR).join(STATE_DIR).join(TOKEN_FILE)
}

fn encode_hex(bytes: &[u8]) -> String {
    bytes.iter().fold(String::new(), |mut acc, b| {
        acc.push_str(&format!("{:02x}", b));
        acc
    })
}

pub fn generate_and_persist(project_root: &Path) -> anyhow::Result<String> {
    let mut buf = [0u8; TOKEN_BYTES];
    getrandom::getrandom(&mut buf)
        .map_err(|e| anyhow::anyhow!("failed to gather entropy for web token: {e}"))?;
    let token = encode_hex(&buf);

    let path = token_path(project_root);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let mut file = std::fs::File::create(&path)?;
    file.write_all(token.as_bytes())?;
    file.flush()?;
    restrict_permissions(&path)?;

    tracing::info!("Web API token written to {}", path.display());
    Ok(token)
}

#[cfg(unix)]
fn restrict_permissions(path: &Path) -> anyhow::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let mut perms = std::fs::metadata(path)?.permissions();
    perms.set_mode(TOKEN_MODE);
    std::fs::set_permissions(path, perms)?;
    Ok(())
}

#[cfg(not(unix))]
fn restrict_permissions(_path: &Path) -> anyhow::Result<()> {
    Ok(())
}
