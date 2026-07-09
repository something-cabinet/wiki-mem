use std::sync::Arc;
use tracing;

use serde_json::json;

use crate::engine::EngineState;
use crate::error::ToolError;
use crate::mcp::handler::ToolArgs;
use crate::mcp::transport::ToolRegistry;

/// Register model tool handlers
pub fn register(registry: &mut ToolRegistry, engine: Arc<EngineState>) {
    let e = engine.clone();
    registry.register_with_schema(
        "model.list",
        "List cached and available models",
        json!({
            "type": "object",
            "properties": {}
        }),
        Arc::new(move |_params| {
            let model_name = e.embedder.model_name().to_string();
            let loaded = e.embedder.is_loaded();
            let indexed = e.vector_store.snapshot().len();

            let home = std::env::var("HOME")
                .or_else(|_| std::env::var("USERPROFILE"))
                .unwrap_or_else(|_| ".".into());
            let models_dir = std::path::PathBuf::from(home).join(".wm").join("models");
            let mut cached_models = Vec::new();
            if models_dir.exists() {
                if let Ok(entries) = std::fs::read_dir(&models_dir) {
                    for entry in entries {
                        match entry {
                            Ok(entry) => {
                                if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                                    let name = entry.file_name().to_string_lossy().to_string();
                                    cached_models.push(serde_json::json!({
                                        "name": name,
                                        "cached": true,
                                        "active": name == model_name && loaded,
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
        }),
    );

    let e = engine.clone();
    registry.register_with_schema(
        "model.status",
        "Show current model state",
        json!({
            "type": "object",
            "properties": {}
        }),
        Arc::new(move |_params| {
            Ok(serde_json::json!({
                "model": e.embedder.model_name(),
                "loaded": e.embedder.is_loaded(),
                "dimensions": e.embedder.output_dim(),
                "sections_indexed": e.vector_store.snapshot().len(),
            }))
        }),
    );

    registry.register_with_schema(
        "model.download",
        "Download an embedding model",
        json!({
            "type": "object",
            "properties": {
                "name": { "type": "string", "description": "Model name (e.g. bge-small-en-v1.5)" }
            },
            "required": ["name"]
        }),
        Arc::new(|params| {
            let args = ToolArgs::new(params);
            #[cfg(feature = "embed")]
            {
                let name = args.require_string("name")?;
                let home = std::env::var("HOME")
                    .or_else(|_| std::env::var("USERPROFILE"))
                    .unwrap_or_else(|_| ".".into());
                let models_dir = std::path::PathBuf::from(home).join(".wm").join("models");
                match crate::onnx::download_model(&name, &models_dir) {
                    Ok(dir) => Ok(serde_json::json!({
                        "status": "ok",
                        "message": format!("Model downloaded to {}", dir.display()),
                        "model_name": name,
                    })),
                    Err(e) => Err(ToolError::internal(format!("Download failed: {}", e))),
                }
            }

            #[cfg(not(feature = "embed"))]
            {
                let _name = args.require_string("name")?;
                Err(ToolError::internal(
                    "Model download requires the 'embed' feature. Rebuild with --features embed.",
                ))
            }
        }),
    );

    registry.register_with_schema(
        "model.remove",
        "Remove a cached model",
        json!({
            "type": "object",
            "properties": {
                "name": { "type": "string", "description": "Model name to remove" }
            },
            "required": ["name"]
        }),
        Arc::new(|params| {
            let args = ToolArgs::new(params);
            let name = args.require_string("name")?;

            let home = std::env::var("HOME")
                .or_else(|_| std::env::var("USERPROFILE"))
                .unwrap_or_else(|_| ".".into());
            let model_dir = std::path::PathBuf::from(home)
                .join(".wm")
                .join("models")
                .join(&name);

            if model_dir.exists() {
                std::fs::remove_dir_all(&model_dir)
                    .map_err(|e| ToolError::internal(format!("Failed to remove model: {}", e)))?;
            }

            Ok(serde_json::json!({
                "status": "removed",
                "model_name": name,
            }))
        }),
    );
}
