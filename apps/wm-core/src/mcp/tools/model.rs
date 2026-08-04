use crate::mcp::prelude::*;
use tracing;
use wm_constants::*;

const MODEL_NAME_SEGMENTS: usize = 1;
const ERR_BAD_MODEL_NAME: &str = "Invalid model name: must be a single path segment";

#[derive(Deserialize, JsonSchema)]
#[serde(tag = "action", rename_all = "snake_case")]
enum WmModelAction {
    List {},
    Status {},
    Download {
        #[schemars(description = "Model name (e.g. bge-small-en-v1.5)")]
        name: String,
    },
    Remove {
        #[schemars(description = "Model name to remove")]
        name: String,
    },
}

pub fn register(registry: &mut ToolRegistry, engine: Arc<EngineState>) {
    registry.register_typed(
        "wm_model",
        "Manage models: list, status, download, remove",
        move |input: WmModelAction| match input {
            WmModelAction::List {} => {
                let model_name = engine.embedder.model_name().to_string();
                let loaded = engine.embedder.is_loaded();
                let indexed = engine.vector_store.snapshot().len();

                let home = std::env::var("HOME")
                    .or_else(|_| std::env::var("USERPROFILE"))
                    .unwrap_or_else(|_| ".".into());
                let models_dir = std::path::PathBuf::from(home).join(WM_DIR).join("models");
                let mut cached_models = Vec::new();
                if models_dir.exists() {
                    if let Ok(entries) = std::fs::read_dir(&models_dir) {
                        for entry in entries {
                            match entry {
                                Ok(entry) => {
                                    if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                                        let mname = entry.file_name().to_string_lossy().to_string();
                                        cached_models.push(serde_json::json!({
                                            "name": mname,
                                            "cached": true,
                                            "active": mname == model_name && loaded,
                                        }));
                                    }
                                }
                                Err(err) => {
                                    tracing::warn!("Failed to read model dir entry: {}", err);
                                }
                            }
                        }
                    }
                }

                Ok(serde_json::json!({
                    "models": cached_models,
                    "active_model": model_name,
                    "loaded": loaded,
                    "sections_indexed": indexed,
                    "available_remote": [
                        {"name": "bge-small-en-v1.5", "dim": 384, "size_mb": 134},
                        {"name": "bge-base-en-v1.5", "dim": 768, "size_mb": 438},
                        {"name": "all-MiniLM-L6-v2", "dim": 384, "size_mb": 90},
                    ],
                }))
            }

            WmModelAction::Status {} => Ok(serde_json::json!({
                "model": engine.embedder.model_name(),
                "loaded": engine.embedder.is_loaded(),
                "dimensions": engine.embedder.output_dim(),
                "sections_indexed": engine.vector_store.snapshot().len(),
            })),

            WmModelAction::Download { name } => {
                #[cfg(feature = "onnx")]
                {
                    let home = std::env::var("HOME")
                        .or_else(|_| std::env::var("USERPROFILE"))
                        .unwrap_or_else(|_| ".".into());
                    let models_dir = std::path::PathBuf::from(home).join(WM_DIR).join("models");
                    match crate::embed::download_model(&name, &models_dir) {
                        Ok(dir) => Ok(serde_json::json!({
                            "status": "ok",
                            "message": format!("Model downloaded to {}", dir.display()),
                            "model_name": name,
                        })),
                        Err(e) => Err(ToolError::internal(format!("Download failed: {}", e))),
                    }
                }
                #[cfg(not(feature = "onnx"))]
                {
                    let _ = name;
                    let result: Result<serde_json::Value, ToolError> = Err(ToolError::internal(
                        "Model download requires the 'onnx' feature. Rebuild with --features onnx.",
                    ));
                    result
                }
            }

            WmModelAction::Remove { name } => {
                let home = std::env::var("HOME")
                    .or_else(|_| std::env::var("USERPROFILE"))
                    .unwrap_or_else(|_| ".".into());
                let models_dir = std::path::PathBuf::from(home).join(WM_DIR).join("models");

                let single_segment =
                    std::path::Path::new(&name).components().count() == MODEL_NAME_SEGMENTS;
                if !single_segment {
                    tracing::warn!("Rejected model name with path separators: {}", name);
                    return Err(ToolError::invalid_params(ERR_BAD_MODEL_NAME));
                }
                let model_dir = crate::shared::helpers::path_confine_helper::confine_strict(
                    &models_dir,
                    std::path::Path::new(&name),
                )?;

                if model_dir.exists() {
                    std::fs::remove_dir_all(&model_dir).map_err(|e| {
                        ToolError::internal(format!("Failed to remove model: {}", e))
                    })?;
                }

                Ok(serde_json::json!({
                    "status": "removed",
                    "model_name": name,
                }))
            }
        },
    );
}
