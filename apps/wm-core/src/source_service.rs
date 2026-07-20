use chrono::Utc;
use sha2::{Digest, Sha256};
use std::path::Path;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use tracing::info;

use crate::engine::{EngineState, SourceEntry, SourceState};
use wm_error::{ToolError, ToolResult};

/// Add a raw source file — copy to .wm/sources/, compute hash, create registry entry
pub fn add_source(engine: &Arc<EngineState>, original_path: &str) -> ToolResult<String> {
    let root = engine.project_root.read()
        .map_err(|_| ToolError::lock_poisoned("project_root"))?
        .clone();
    let src_path = Path::new(original_path);
    if !src_path.exists() {
        return Err(ToolError::not_found("file", original_path));
    }

    // Read and hash
    let content = std::fs::read(src_path)
        .map_err(|e| ToolError::internal(format!("Failed to read {}: {}", original_path, e)))?;
    let hash = content_hash(&content);
    let slug = src_path
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "source".to_string());
    let ext = src_path
        .extension()
        .map(|e| format!(".{}", e.to_string_lossy()))
        .unwrap_or_default();

    // Copy to .wm/sources/
    let sources_dir = root.join(".wm").join("sources");
    std::fs::create_dir_all(&sources_dir).ok();
    let stored_name = format!("{}-{}{}", &hash[..8], slug, ext);
    let stored_path = sources_dir.join(&stored_name);
    engine
        .write_channel
        .write(stored_path.clone(), content.to_vec())
        .map_err(|e| ToolError::internal(format!("Write channel error: {}", e)))?;

    let id = format!("src_{}", &hash[..8]);

    let entry = SourceEntry {
        id: id.clone(),
        original_path: Some(original_path.to_string()),
        stored_path,
        content_hash: hash,
        state: SourceState::Pending,
        added_at: Utc::now().to_rfc3339(),
        last_processed_at: None,
        page_refs: Vec::new(),
        page_count: 0,
        error_message: None,
        retry_count: 0,
    };

    engine
        .source_registry
        .write()
        .map_err(|_| ToolError::lock_poisoned("registry"))?
        .insert(id.clone(), entry);
    engine.stale_flag.store(true, Ordering::Release);
    info!("Source added: {} ({})", id, original_path);

    Ok(id)
}

/// Claim a source for processing — CAS transition: pending/stale → processing
pub fn claim_source_and_read_content(engine: &Arc<EngineState>, id: &str) -> ToolResult<String> {
    let mut registry = engine.source_registry.write().map_err(|_| ToolError::lock_poisoned("registry"))?;
    let entry = registry
        .get_mut(id)
        .ok_or_else(|| ToolError::not_found("source", id))?;

    match entry.state {
        SourceState::Pending | SourceState::Stale => {
            entry.state = SourceState::Processing;
            entry.retry_count += 1;
        }
        SourceState::Processing => {
            // Check for orphan (30 min timeout)
            if let Some(ref last) = entry.last_processed_at {
                if let Ok(then) = chrono::DateTime::parse_from_rfc3339(last) {
                    let elapsed = Utc::now().signed_duration_since(then);
                    if elapsed.num_minutes() > 30 {
                        info!(
                            "Orphan source {} auto-reset to pending ({} min stale)",
                            id,
                            elapsed.num_minutes()
                        );
                        entry.state = SourceState::Pending;
                    } else {
                        return Err(ToolError::internal(format!(
                            "Source {} is already being processed by another agent",
                            id
                        )));
                    }
                }
            }
            return Err(ToolError::internal(format!(
                "Source {} is already being processed",
                id
            )));
        }
        SourceState::Done => {
            return Err(ToolError::internal(format!(
                "Source {} already processed. Use source.verify to check for staleness.",
                id
            )));
        }
        SourceState::Error => {
            entry.state = SourceState::Processing;
        }
    }

    entry.last_processed_at = Some(Utc::now().to_rfc3339());

    // Read the source content
    let content = std::fs::read_to_string(&entry.stored_path)
        .map_err(|e| ToolError::internal(format!("Failed to read stored source: {}", e)))?;

    Ok(content)
}

/// Mark source as complete with page references
pub fn complete_source(
    engine: &Arc<EngineState>,
    id: &str,
    page_refs: &[String],
) -> ToolResult<()> {
    let root = engine.project_root.read()
        .map_err(|_| ToolError::lock_poisoned("project_root"))?
        .clone();
    let mut registry = engine.source_registry.write().map_err(|_| ToolError::lock_poisoned("registry"))?;
    let entry = registry
        .get_mut(id)
        .ok_or_else(|| ToolError::not_found("source", id))?;

    if entry.state != SourceState::Processing {
        return Err(ToolError::internal(format!(
            "Source {} is not in processing state",
            id
        )));
    }

    entry.state = SourceState::Done;
    entry.page_refs = page_refs.to_vec();
    entry.page_count = page_refs.len();
    entry.last_processed_at = Some(Utc::now().to_rfc3339());

    // Auto-append to log.md (create if not exists)
    let log_path = root.join(".wm").join("wiki").join("log.md");
    let log_entry = std::fs::read_to_string(&log_path).unwrap_or_default();
    let new_entry = format!(
        "\n{} | source.complete | {} → {} pages",
        Utc::now().to_rfc3339(),
        id,
        page_refs.len()
    );
    engine
        .write_channel
        .write(
            log_path,
            format!("{}{}", log_entry, new_entry).into_bytes(),
        )
        .ok();

    // Mark stale for rebuild
    engine.stale_flag.store(true, Ordering::Release);
    info!("Source completed: {} → {} pages", id, page_refs.len());

    Ok(())
}

/// Mark a source as errored with a message
pub fn error_source(engine: &Arc<EngineState>, id: &str, message: &str) -> ToolResult<()> {
    let mut registry = engine.source_registry.write().map_err(|_| ToolError::lock_poisoned("registry"))?;
    let entry = registry
        .get_mut(id)
        .ok_or_else(|| ToolError::not_found("source", id))?;

    if entry.state != SourceState::Processing {
        return Err(ToolError::internal(format!(
            "Source {} is not in processing state",
            id
        )));
    }

    entry.state = SourceState::Error;
    entry.error_message = Some(message.to_string());
    entry.last_processed_at = Some(Utc::now().to_rfc3339());
    entry.retry_count += 1;
    info!("Source errored: {} — {}", id, message);

    Ok(())
}

/// Verify source staleness — recompute SHA-256 and compare
pub fn verify_source(engine: &Arc<EngineState>, id: &str) -> ToolResult<bool> {
    let (stored_path, stored_hash) = {
        let registry = engine.source_registry.read().map_err(|_| ToolError::lock_poisoned("registry"))?;
        let entry = registry
            .get(id)
            .ok_or_else(|| ToolError::not_found("source", id))?;
        (entry.stored_path.clone(), entry.content_hash.clone())
    };

    let content = match std::fs::read(&stored_path) {
        Ok(c) => c,
        Err(_) => {
            // File missing — mark as stale
            let mut registry = engine.source_registry.write().map_err(|_| ToolError::lock_poisoned("registry"))?;
            if let Some(entry) = registry.get_mut(id) {
                entry.state = SourceState::Stale;
            }
            return Ok(true);
        }
    };
    let current_hash = content_hash(&content);
    let is_stale = current_hash != stored_hash;

    if is_stale {
        info!("Source {} is stale (hash mismatch)", id);
        // Write stale state back to registry
        let mut registry = engine.source_registry.write().map_err(|_| ToolError::lock_poisoned("registry"))?;
        if let Some(entry) = registry.get_mut(id) {
            entry.state = SourceState::Stale;
        }
    }

    Ok(is_stale)
}

/// List sources filtered by state
pub fn list_sources(
    engine: &Arc<EngineState>,
    state_filter: Option<&str>,
) -> ToolResult<Vec<serde_json::Value>> {
    let registry = engine.source_registry.read().map_err(|_| ToolError::lock_poisoned("registry"))?;
    let sources: Vec<serde_json::Value> = registry
        .values()
        .filter(|entry| match state_filter {
            Some("pending") => matches!(entry.state, SourceState::Pending),
            Some("processing") => matches!(entry.state, SourceState::Processing),
            Some("done") => matches!(entry.state, SourceState::Done),
            Some("error") => matches!(entry.state, SourceState::Error),
            Some("stale") => matches!(entry.state, SourceState::Stale),
            Some(_) | None => true,
        })
        .map(|entry| {
            serde_json::json!({
                "id": entry.id,
                "state": format!("{:?}", entry.state).to_lowercase(),
                "original_path": entry.original_path,
                "page_count": entry.page_count,
                "page_refs": entry.page_refs,
                "added_at": entry.added_at,
                "last_processed_at": entry.last_processed_at,
            })
        })
        .collect();

    Ok(sources)
}

/// Discover new sources in configured directories
pub fn discover_sources(
    engine: &Arc<EngineState>,
    dirs: &[String],
    extensions: Option<&[String]>,
) -> ToolResult<Vec<String>> {
    let mut discovered = Vec::new();

    for dir in dirs {
        let dir_path = Path::new(dir);
        if !dir_path.exists() {
            continue;
        }

        let entries = walkdir::WalkDir::new(dir_path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .filter(|e| {
                if let Some(exts) = extensions {
                    if exts.is_empty() {
                        return true;
                    }
                    e.path()
                        .extension()
                        .and_then(|ext| ext.to_str())
                        .map(|ext| exts.iter().any(|allowed| allowed == ext))
                        .unwrap_or(false)
                } else {
                    true
                }
            });

        for entry in entries {
            let path = entry.path();
            let content = match std::fs::read(path) {
                Ok(c) => c,
                Err(_) => continue,
            };
            let hash = content_hash(&content);

            // Check if already tracked
            let already_tracked = {
                let registry = engine.source_registry.read().map_err(|_| ToolError::lock_poisoned("registry"))?;
                registry.values().any(|e| {
                    e.content_hash == hash
                        || e.original_path.as_deref() == Some(&path.to_string_lossy())
                })
            };

            if !already_tracked {
                if let Ok(id) = add_source(engine, &path.to_string_lossy()) {
                    discovered.push(id);
                }
            }
        }
    }

    Ok(discovered)
}

/// Remove a source entry from the registry (does not delete the file)
pub fn remove_source(engine: &Arc<EngineState>, id: &str) -> ToolResult<()> {
    let mut registry = engine.source_registry.write().map_err(|_| ToolError::lock_poisoned("registry"))?;
    registry
        .remove(id)
        .ok_or_else(|| ToolError::not_found("source", id))?;
    Ok(())
}

/// Get detailed status for a source
pub fn source_status(engine: &Arc<EngineState>, id: &str) -> ToolResult<serde_json::Value> {
    let registry = engine.source_registry.read().map_err(|_| ToolError::lock_poisoned("registry"))?;
    let entry = registry
        .get(id)
        .ok_or_else(|| ToolError::not_found("source", id))?;
    Ok(serde_json::json!({
        "id": entry.id,
        "state": format!("{:?}", entry.state).to_lowercase(),
        "original_path": entry.original_path,
        "stored_path": entry.stored_path.to_string_lossy().to_string(),
        "content_hash": entry.content_hash,
        "added_at": entry.added_at,
        "last_processed_at": entry.last_processed_at,
        "page_refs": entry.page_refs,
        "page_count": entry.page_count,
        "error_message": entry.error_message,
        "retry_count": entry.retry_count,
    }))
}

fn content_hash(content: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content);
    hex::encode(hasher.finalize())
}


