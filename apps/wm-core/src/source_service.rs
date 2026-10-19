use chrono::Utc;
use sha2::{Digest, Sha256};
use std::path::Path;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use tracing::info;

use wm_constants::*;

use crate::engine::{EngineState, SourceEntry, SourceState};
use crate::error::{ToolError, ToolResult};

const ERR_SOURCE_NOT_ALLOWED: &str =
    "Access denied: path is not under a configured source_dirs entry";
const ERR_SOURCE_EXT: &str = "Access denied: file extension is not in source_extensions";
const CLAIM_TIMEOUT_MINUTES: i64 = 30;

fn extension_allowed(candidate: &Path, exts: &[String]) -> bool {
    if exts.is_empty() {
        return true;
    }
    candidate
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| exts.iter().any(|allowed| allowed == e))
        .unwrap_or(false)
}

fn source_dir_allows(root: &Path, dir: &str, candidate: &Path) -> bool {
    let base = root.join(dir);
    let relative = candidate
        .strip_prefix(&base)
        .or_else(|_| candidate.strip_prefix(dir));
    let Ok(rel) = relative else {
        return false;
    };
    crate::shared::helpers::path_confine_helper::confine_strict(&base, rel).is_ok()
}

pub fn add_source(engine: &Arc<EngineState>, original_path: &str) -> ToolResult<String> {
    let root = engine
        .project_root
        .read()
        .map_err(|_| ToolError::lock_poisoned("project_root"))?
        .clone();

    let (dirs, exts) = {
        let cfg = engine
            .config
            .read()
            .map_err(|_| ToolError::lock_poisoned("config"))?;
        (cfg.source_dirs.clone(), cfg.source_extensions.clone())
    };

    let src_path = Path::new(original_path);

    if !extension_allowed(src_path, &exts) {
        tracing::warn!(
            "Rejected source with disallowed extension: {}",
            original_path
        );
        return Err(ToolError::invalid_params(ERR_SOURCE_EXT));
    }

    if !dirs.iter().any(|d| source_dir_allows(&root, d, src_path)) {
        tracing::warn!("Rejected source outside source_dirs: {}", original_path);
        return Err(ToolError::invalid_params(ERR_SOURCE_NOT_ALLOWED));
    }

    if !src_path.exists() {
        return Err(ToolError::not_found("file", original_path));
    }

    let content = std::fs::read(src_path)
        .map_err(|e| ToolError::internal(format!("Failed to read {}: {}", original_path, e)))?;
    let hash = content_hash(&content);
    let slug = src_path
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "source".into());
    let ext = src_path
        .extension()
        .map(|e| format!(".{}", e.to_string_lossy()))
        .unwrap_or_default();

    let sources_dir = root.join(WM_DIR).join(SOURCES_DIR);
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

    engine.source_registry.insert(entry)?;
    engine.stale_flag.store(true, Ordering::Release);
    info!("Source added: {} ({})", id, original_path);

    Ok(id)
}

pub fn claim_source_and_read_content(engine: &Arc<EngineState>, id: &str) -> ToolResult<String> {
    engine.source_registry.update(id, |entry| {
        match entry.state {
            SourceState::Pending | SourceState::Stale => {
                entry.state = SourceState::Processing;
                entry.retry_count = entry.retry_count.wrapping_add(1);
            }
            SourceState::Processing => {
                if let Some(ref last) = entry.last_processed_at {
                    if let Ok(then) = chrono::DateTime::parse_from_rfc3339(last) {
                        let elapsed = Utc::now().signed_duration_since(then);
                        if elapsed.num_minutes() > CLAIM_TIMEOUT_MINUTES {
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

        std::fs::read_to_string(&entry.stored_path)
            .map_err(|e| ToolError::internal(format!("Failed to read stored source: {}", e)))
    })
}

pub fn complete_source(
    engine: &Arc<EngineState>,
    id: &str,
    page_refs: &[String],
) -> ToolResult<()> {
    let root = engine
        .project_root
        .read()
        .map_err(|_| ToolError::lock_poisoned("project_root"))?
        .clone();
    let log_path = root.join(WM_DIR).join(WIKI_DIR).join("log.md");

    engine.source_registry.update(id, |entry| {
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

        let log_entry = std::fs::read_to_string(&log_path).unwrap_or_default();
        let new_entry = format!(
            "\n{} | source.complete | {} → {} pages",
            Utc::now().to_rfc3339(),
            id,
            page_refs.len()
        );
        engine
            .write_channel
            .write(log_path, format!("{}{}", log_entry, new_entry).into_bytes())
            .ok();

        Ok(())
    })?;

    engine.stale_flag.store(true, Ordering::Release);
    info!("Source completed: {} → {} pages", id, page_refs.len());

    Ok(())
}

pub fn error_source(engine: &Arc<EngineState>, id: &str, message: &str) -> ToolResult<()> {
    engine.source_registry.update(id, |entry| {
        if entry.state != SourceState::Processing {
            return Err(ToolError::internal(format!(
                "Source {} is not in processing state",
                id
            )));
        }

        entry.state = SourceState::Error;
        entry.error_message = Some(message.to_string());
        entry.last_processed_at = Some(Utc::now().to_rfc3339());
        entry.retry_count = entry.retry_count.wrapping_add(1);

        Ok(())
    })?;

    info!("Source errored: {} — {}", id, message);

    Ok(())
}

pub fn verify_source(engine: &Arc<EngineState>, id: &str) -> ToolResult<bool> {
    let (stored_path, stored_hash) = engine
        .source_registry
        .get(id)
        .map(|entry| (entry.stored_path, entry.content_hash))?;

    let content = match std::fs::read(&stored_path) {
        Ok(c) => c,
        Err(_) => {
            mark_stale(engine, id)?;
            return Ok(true);
        }
    };
    let current_hash = content_hash(&content);
    let is_stale = current_hash != stored_hash;

    if is_stale {
        info!("Source {} is stale (hash mismatch)", id);
        mark_stale(engine, id)?;
    }

    Ok(is_stale)
}

fn mark_stale(engine: &Arc<EngineState>, id: &str) -> ToolResult<()> {
    engine.source_registry.update_if_present(id, |entry| {
        entry.state = SourceState::Stale;
    })?;
    Ok(())
}

pub fn list_sources(
    engine: &Arc<EngineState>,
    state_filter: Option<&str>,
) -> ToolResult<Vec<serde_json::Value>> {
    let entries = engine.source_registry.entries()?;
    let sources: Vec<serde_json::Value> = entries
        .iter()
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

            let already_tracked = engine
                .source_registry
                .is_tracked(&hash, path.to_string_lossy().as_ref())?;

            if !already_tracked {
                if let Ok(id) = add_source(engine, &path.to_string_lossy()) {
                    discovered.push(id);
                }
            }
        }
    }

    Ok(discovered)
}

pub fn remove_source(engine: &Arc<EngineState>, id: &str) -> ToolResult<()> {
    engine.source_registry.remove(id)
}

pub fn source_status(engine: &Arc<EngineState>, id: &str) -> ToolResult<serde_json::Value> {
    let entry = engine.source_registry.get(id)?;
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
