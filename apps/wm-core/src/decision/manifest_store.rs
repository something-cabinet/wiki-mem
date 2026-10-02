use std::path::{Path, PathBuf};

use wm_constants::WM_DIR;
use wm_engine::{DecisionError, ModelManifest};

pub const DEFAULT_DECISION_MODEL: &str = "gliner2.5-small-v1";
pub const MODELS_MANIFEST_FILE: &str = "models.json";
pub const MODELS_DIR: &str = "models";

pub fn manifest_path(project_root: &Path) -> PathBuf {
    project_root.join(WM_DIR).join(MODELS_MANIFEST_FILE)
}

pub fn load_manifest(project_root: &Path) -> Result<ModelManifest, DecisionError> {
    let path = manifest_path(project_root);
    let raw = std::fs::read_to_string(&path)
        .map_err(|error| DecisionError::InvalidManifest(format!("{}: {error}", path.display())))?;
    ModelManifest::parse(&raw)
}

pub fn models_cache_dir() -> PathBuf {
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .unwrap_or_else(|_| ".".to_owned());
    PathBuf::from(home).join(WM_DIR).join(MODELS_DIR)
}

pub fn cached_model_dir(name: &str) -> PathBuf {
    models_cache_dir().join(name)
}

pub fn is_cached(name: &str) -> bool {
    cached_model_dir(name).is_dir()
}
