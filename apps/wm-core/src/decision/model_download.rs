use std::io::Read;
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};
use wm_engine::{source_repo, DecisionError, ModelEntry, ModelFile};

pub const HF_RESOLVE_BASE: &str = "https://huggingface.co";
const READ_BUFFER_BYTES: usize = 8192;

/// Fetch every manifest file for `entry` into `models_dir/<name>/`, verifying
/// the pinned SHA-256 (and size when non-zero). Already-verified files are
/// reused. Reusable from `wm_model`/the CLI.
pub fn ensure_model(entry: &ModelEntry, models_dir: &Path) -> Result<PathBuf, DecisionError> {
    let repo = source_repo(&entry.source).ok_or_else(|| {
        DecisionError::InvalidManifest(format!(
            "model '{}' source is not an hf: reference",
            entry.name
        ))
    })?;
    let model_dir = models_dir.join(&entry.name);
    std::fs::create_dir_all(&model_dir).map_err(|error| DecisionError::Backend {
        detail: format!("create {}: {error}", model_dir.display()),
    })?;
    for file in &entry.files {
        ensure_file(repo, &entry.revision, file, &model_dir)?;
    }
    Ok(model_dir)
}

fn ensure_file(
    repo: &str,
    revision: &str,
    file: &ModelFile,
    model_dir: &Path,
) -> Result<(), DecisionError> {
    if file.has_placeholder_hash() {
        return Err(DecisionError::InvalidManifest(format!(
            "model file '{}' still has a placeholder sha256",
            file.path
        )));
    }
    let target = model_dir.join(&file.path);
    if target.is_file() {
        let digest = sha256_file(&target)?;
        if digest == file.sha256 {
            return Ok(());
        }
        std::fs::remove_file(&target).map_err(|error| DecisionError::Backend {
            detail: format!("remove stale {}: {error}", target.display()),
        })?;
    }
    if let Some(parent) = target.parent() {
        std::fs::create_dir_all(parent).map_err(|error| DecisionError::Backend {
            detail: format!("create {}: {error}", parent.display()),
        })?;
    }

    let url = format!("{HF_RESOLVE_BASE}/{repo}/resolve/{revision}/{}", file.path);
    let bytes = fetch(&url)?;
    let digest = hex::encode(Sha256::digest(&bytes));
    if digest != file.sha256 {
        return Err(DecisionError::InvalidManifest(format!(
            "sha256 mismatch for '{}': got {digest}, expected {}",
            file.path, file.sha256
        )));
    }
    if file.size_bytes != 0 {
        let actual = u64::try_from(bytes.len()).unwrap_or(u64::MAX);
        if actual != file.size_bytes {
            return Err(DecisionError::InvalidManifest(format!(
                "size mismatch for '{}': got {actual}, expected {}",
                file.path, file.size_bytes
            )));
        }
    }
    std::fs::write(&target, &bytes).map_err(|error| DecisionError::Backend {
        detail: format!("write {}: {error}", target.display()),
    })
}

fn fetch(url: &str) -> Result<Vec<u8>, DecisionError> {
    let response = ureq::get(url)
        .call()
        .map_err(|error| DecisionError::Backend {
            detail: format!("request {url}: {error}"),
        })?;
    let mut bytes = Vec::new();
    response
        .into_reader()
        .read_to_end(&mut bytes)
        .map_err(|error| DecisionError::Backend {
            detail: format!("read {url}: {error}"),
        })?;
    Ok(bytes)
}

fn sha256_file(path: &Path) -> Result<String, DecisionError> {
    let mut file = std::fs::File::open(path).map_err(|error| DecisionError::Backend {
        detail: format!("open {}: {error}", path.display()),
    })?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; READ_BUFFER_BYTES];
    loop {
        let read = file
            .read(&mut buffer)
            .map_err(|error| DecisionError::Backend {
                detail: format!("read {}: {error}", path.display()),
            })?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(hex::encode(hasher.finalize()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use wm_engine::ModelFile;

    fn file(path: &str, sha256: &str, size_bytes: u64) -> ModelFile {
        ModelFile {
            path: path.to_owned(),
            sha256: sha256.to_owned(),
            size_bytes,
        }
    }

    #[test]
    fn rejects_placeholder_hash_without_downloading() {
        let dir = tempfile::tempdir().expect("tempdir");
        let entry = file(
            "model.safetensors",
            "0000000000000000000000000000000000000000000000000000000000000000",
            0,
        );
        let error = ensure_file("fastino/gliner2.5-small-v1", "rev", &entry, dir.path())
            .expect_err("placeholder must fail");
        assert!(matches!(error, DecisionError::InvalidManifest(_)));
    }

    #[test]
    fn reuses_a_verified_local_file_without_network() {
        let dir = tempfile::tempdir().expect("tempdir");
        let target = dir.path().join("config.json");
        std::fs::write(&target, b"{\"a\":1}").expect("write fixture");
        let digest = sha256_file(&target).expect("hash fixture");
        let entry = file("config.json", &digest, 7);

        ensure_file("fastino/gliner2.5-small-v1", "rev", &entry, dir.path())
            .expect("verified local file must be reused");
        assert_eq!(std::fs::read_to_string(&target).unwrap(), "{\"a\":1}");
    }

    #[test]
    fn hashes_file_contents_as_sha256_hex() {
        let dir = tempfile::tempdir().expect("tempdir");
        let target = dir.path().join("blob.bin");
        std::fs::write(&target, b"abc").expect("write fixture");
        assert_eq!(
            sha256_file(&target).expect("hash"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }
}
