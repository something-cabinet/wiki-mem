use chrono::Utc;
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use std::sync::atomic::Ordering;
use std::sync::Arc;
use tracing::info;

use crate::engine::{EngineState, SourceEntry, SourceState};
use crate::error::{ToolError, ToolResult};

/// Add a raw source file — copy to .wm/sources/, compute hash, create registry entry
pub fn add_source(engine: &Arc<EngineState>, original_path: &str) -> ToolResult<String> {
    let src_path = Path::new(original_path);
    if !src_path.exists() {
        return Err(ToolError::not_found("file", original_path));
    }

    // Read and hash
    let content = std::fs::read(src_path)
        .map_err(|e| ToolError::internal(format!("Failed to read {}: {}", original_path, e)))?;
    let hash = content_hash(&content);
    let slug = src_path.file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "source".to_string());
    let ext = src_path.extension()
        .map(|e| format!(".{}", e.to_string_lossy()))
        .unwrap_or_default();

    // Copy to .wm/sources/
    let sources_dir = Path::new(".wm").join("sources");
    std::fs::create_dir_all(&sources_dir).ok();
    let stored_name = format!("{}-{}{}", &hash[..8], slug, ext);
    let stored_path = sources_dir.join(&stored_name);
    std::fs::write(&stored_path, &content)
        .map_err(|e| ToolError::internal(format!("Failed to copy source: {}", e)))?;

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

    engine.source_registry.write().unwrap().insert(id.clone(), entry);
    engine.stale_flag.store(true, Ordering::Release);
    info!("Source added: {} ({})", id, original_path);

    Ok(id)
}

/// Claim a source for processing — CAS transition: pending/stale → processing
pub fn process_source(engine: &Arc<EngineState>, id: &str) -> ToolResult<String> {
    let mut registry = engine.source_registry.write().unwrap();
    let entry = registry.get_mut(id)
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
                        info!("Orphan source {} auto-reset to pending ({} min stale)", id, elapsed.num_minutes());
                        entry.state = SourceState::Pending;
                    } else {
                        return Err(ToolError::internal(format!("Source {} is already being processed by another agent", id)));
                    }
                }
            }
            return Err(ToolError::internal(format!("Source {} is already being processed", id)));
        }
        SourceState::Done => {
            return Err(ToolError::internal(format!("Source {} already processed. Use source.verify to check for staleness.", id)));
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
pub fn complete_source(engine: &Arc<EngineState>, id: &str, page_refs: &[String]) -> ToolResult<()> {
    let mut registry = engine.source_registry.write().unwrap();
    let entry = registry.get_mut(id)
        .ok_or_else(|| ToolError::not_found("source", id))?;

    if entry.state != SourceState::Processing {
        return Err(ToolError::internal(format!("Source {} is not in processing state", id)));
    }

    entry.state = SourceState::Done;
    entry.page_refs = page_refs.to_vec();
    entry.page_count = page_refs.len();
    entry.last_processed_at = Some(Utc::now().to_rfc3339());

    // Auto-append to log.md
    if let Ok(log_entry) = std::fs::read_to_string(".wm/wiki/log.md") {
        let new_entry = format!("\n{} | source.complete | {} → {} pages", Utc::now().to_rfc3339(), id, page_refs.len());
        std::fs::write(".wm/wiki/log.md", format!("{}{}", log_entry, new_entry)).ok();
    }

    // Mark stale for rebuild
    engine.stale_flag.store(true, Ordering::Release);
    info!("Source completed: {} → {} pages", id, page_refs.len());

    Ok(())
}

/// Verify source staleness — recompute SHA-256 and compare
pub fn verify_source(engine: &Arc<EngineState>, id: &str) -> ToolResult<bool> {
    let registry = engine.source_registry.read().unwrap();
    let entry = registry.get(id)
        .ok_or_else(|| ToolError::not_found("source", id))?;

    let content = match std::fs::read(&entry.stored_path) {
        Ok(c) => c,
        Err(_) => return Ok(true), // file missing = stale
    };
    let current_hash = content_hash(&content);
    let is_stale = current_hash != entry.content_hash;

    if is_stale {
        info!("Source {} is stale (hash mismatch)", id);
    }

    Ok(is_stale)
}

/// List sources filtered by state
pub fn list_sources(engine: &Arc<EngineState>, state_filter: Option<&str>) -> ToolResult<Vec<serde_json::Value>> {
    let registry = engine.source_registry.read().unwrap();
    let sources: Vec<serde_json::Value> = registry.values().filter(|entry| {
        match state_filter {
            Some("pending") => matches!(entry.state, SourceState::Pending),
            Some("processing") => matches!(entry.state, SourceState::Processing),
            Some("done") => matches!(entry.state, SourceState::Done),
            Some("error") => matches!(entry.state, SourceState::Error),
            Some("stale") => matches!(entry.state, SourceState::Stale),
            Some(_) | None => true,
        }
    }).map(|entry| {
        serde_json::json!({
            "id": entry.id,
            "state": format!("{:?}", entry.state).to_lowercase(),
            "original_path": entry.original_path,
            "page_count": entry.page_count,
            "page_refs": entry.page_refs,
            "added_at": entry.added_at,
            "last_processed_at": entry.last_processed_at,
        })
    }).collect();

    Ok(sources)
}

/// Discover new sources in configured directories
pub fn discover_sources(engine: &Arc<EngineState>, dirs: &[String]) -> ToolResult<Vec<String>> {
    let mut discovered = Vec::new();

    for dir in dirs {
        let dir_path = Path::new(dir);
        if !dir_path.exists() { continue; }

        let entries = walkdir::WalkDir::new(dir_path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file());

        for entry in entries {
            let path = entry.path();
            let content = match std::fs::read(path) {
                Ok(c) => c,
                Err(_) => continue,
            };
            let hash = content_hash(&content);

            // Check if already tracked
            let already_tracked = {
                let registry = engine.source_registry.read().unwrap();
                registry.values().any(|e| {
                    e.content_hash == hash || e.original_path.as_deref() == Some(&path.to_string_lossy())
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

fn content_hash(content: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content);
    hex::encode(hasher.finalize())
}

/// Check for orphan timers on startup — any time_started > 24h
pub fn recover_orphan_timers(engine: &Arc<EngineState>) -> ToolResult<usize> {
    let mut count = 0;
    let snapshot = engine.graph.load();
    let graph = &snapshot.0;
    let index = &snapshot.1;

    for (page_id, node_idx) in index {
        let _meta = &graph[*node_idx];
        if page_id.starts_with("wiki:tasks:") {
            // Read page frontmatter, check for time_started
            let path = resolve_page_path(page_id);
            if let Ok(content) = std::fs::read_to_string(&path) {
                let (_fm, _) = crate::parser::extract_frontmatter(&content);
                // In a full implementation, check time_started from frontmatter
                // For now, log and count
                count += 1;
            }
        }
    }

    Ok(count)
}

fn resolve_page_path(id: &str) -> PathBuf {
    let path_part = id.replace(':', "/");
    PathBuf::from(".wm").join("wiki").join(format!("{}.md", path_part))
}
