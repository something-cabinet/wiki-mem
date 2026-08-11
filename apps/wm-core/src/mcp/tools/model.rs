use crate::mcp::prelude::*;
use tracing;
use wm_constants::*;

const MODEL_NAME_SEGMENTS: usize = 1;
const ERR_BAD_MODEL_NAME: &str = "Invalid model name: must be a single path segment";
#[cfg(feature = "onnx")]
const ERR_UNKNOWN_MODEL: &str = "Unknown model name";

/// Canonical registry of models the tool may download or remove. Single source
/// of truth — both the `list` payload and the `remove` allowlist read from it,
/// so a name accepted for download is always removable and vice versa.
pub const MODEL_REGISTRY: &[&str] = &[
    "bge-small-en-v1.5",
    "bge-base-en-v1.5",
    "all-MiniLM-L6-v2",
];

fn models_dir() -> std::path::PathBuf {
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .unwrap_or_else(|_| ".".into());
    std::path::PathBuf::from(home).join(WM_DIR).join("models")
}

fn audit_reject(engine: &EngineState, name: &str, detail: &str) {
    use crate::shared::audit_sink::{self, SecurityAuditEvent};
    let project_root = engine
        .project_root
        .read()
        .map(|r| r.clone())
        .unwrap_or_default();
    audit_sink::write_security_audit(
        &project_root,
        &SecurityAuditEvent {
            timestamp: chrono::Utc::now().to_rfc3339(),
            category: "security".into(),
            kind: audit_sink::KIND_INVALID_MODEL.into(),
            tool: "wm_model".into(),
            detail: audit_sink::sanitize(detail),
            path: audit_sink::sanitize(name),
        },
    );
}

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
    registry.register_typed_async(
        "wm_model",
        "Manage models: list, status, download, remove",
        move |input: WmModelAction| {
            let engine = engine.clone();
            async move {
                match input {
                    WmModelAction::List {} => {
                        let model_name = engine.embedder.model_name().to_string();
                        let loaded = engine.embedder.is_loaded();
                        let indexed = engine.vector_store.snapshot().len();

                        let md = models_dir();
                        let list_model = model_name.clone();
                        let cached_models: Vec<serde_json::Value> =
                            tokio::task::spawn_blocking(move || {
                                let mut out = Vec::new();
                                if md.exists() {
                                    if let Ok(entries) = std::fs::read_dir(&md) {
                                        for entry in entries.flatten() {
                                            if entry
                                                .file_type()
                                                .map(|t| t.is_dir())
                                                .unwrap_or(false)
                                            {
                                                let mname =
                                                    entry.file_name().to_string_lossy().to_string();
                                                out.push(serde_json::json!({
                                                    "name": mname,
                                                    "cached": true,
                                                    "active": mname == list_model && loaded,
                                                }));
                                            }
                                        }
                                    }
                                }
                                out
                            })
                            .await
                            .map_err(|e| {
                                ToolError::internal(format!("model list task failed: {e}"))
                            })?;

                        let metadata = [
                            ("bge-small-en-v1.5", (384, 134)),
                            ("bge-base-en-v1.5", (768, 438)),
                            ("all-MiniLM-L6-v2", (384, 90)),
                        ];
                        let available_remote: Vec<serde_json::Value> = MODEL_REGISTRY
                            .iter()
                            .map(|name| {
                                let (dim, size_mb) = metadata
                                    .iter()
                                    .find(|(n, _)| *n == *name)
                                    .map(|(_, m)| *m)
                                    .unwrap_or((0, 0));
                                serde_json::json!({
                                    "name": name,
                                    "dim": dim,
                                    "size_mb": size_mb,
                                })
                            })
                            .collect();

                        Ok(serde_json::json!({
                            "models": cached_models,
                            "active_model": model_name,
                            "loaded": loaded,
                            "sections_indexed": indexed,
                            "available_remote": available_remote,
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
                            if !MODEL_REGISTRY.contains(&name.as_str()) {
                                return Err(ToolError::invalid_params(format!(
                                    "{ERR_UNKNOWN_MODEL}: {name}"
                                )));
                            }
                            let md = models_dir();
                            let name_c = name.clone();
                            let result = tokio::task::spawn_blocking(move || {
                                crate::embed::download_model(&name_c, &md)
                            })
                            .await
                            .map_err(|e| {
                                ToolError::internal(format!("model download task failed: {e}"))
                            })?;
                            match result {
                                Ok(dir) => Ok(serde_json::json!({
                                    "status": "ok",
                                    "message": format!("Model downloaded to {}", dir.display()),
                                    "model_name": name,
                                })),
                                Err(e) => Err(ToolError::internal(format!(
                                    "Download failed: {}",
                                    e
                                ))),
                            }
                        }
                        #[cfg(not(feature = "onnx"))]
                        {
                            let _ = name;
                            Err(ToolError::internal(
                                "Model download requires the 'onnx' feature. Rebuild with --features onnx.",
                            ))
                        }
                    }

                    WmModelAction::Remove { name } => {
                        let single_segment = std::path::Path::new(&name)
                            .components()
                            .count()
                            == MODEL_NAME_SEGMENTS;
                        if !single_segment {
                            tracing::warn!(
                                "Rejected model name with path separators: {}",
                                name
                            );
                            audit_reject(&engine, &name, "model name is not a single path segment");
                            return Err(ToolError::invalid_params(ERR_BAD_MODEL_NAME));
                        }
                        if !MODEL_REGISTRY.contains(&name.as_str()) {
                            tracing::warn!("Rejected unknown model name for removal: {}", name);
                            audit_reject(&engine, &name, "unknown model name");
                            return Err(ToolError::not_found("model", &name));
                        }
                        let md = models_dir();
                        let model_dir = crate::shared::helpers::path_confine_helper::confine_strict(
                            &md,
                            std::path::Path::new(&name),
                        )?;

                        if model_dir.exists() {
                            tokio::fs::remove_dir_all(&model_dir)
                                .await
                                .map_err(|e| {
                                    ToolError::internal(format!("Failed to remove model: {e}"))
                                })?;
                        }

                        Ok(serde_json::json!({
                            "status": "removed",
                            "model_name": name,
                        }))
                    }
                }
            }
        },
    );
}
