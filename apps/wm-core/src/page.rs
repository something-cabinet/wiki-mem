use std::path::{Path, PathBuf};
use std::sync::atomic::Ordering;
use std::sync::Arc;

use crate::engine::{AcceptanceCriterion, EngineState, PageType, WikiPageContent};
use crate::error::{ToolError, ToolResult};
use crate::parser::{self, parse_wiki_page};

/// Create a new wiki page
pub fn create_page(
    engine: &Arc<EngineState>,
    path: &str,
    frontmatter: &str,
    content: &str,
) -> ToolResult<String> {
    let full_path = resolve_page_path(&engine.config.read().map_err(|_| ToolError::lock_poisoned("config"))?.project_name, path)?;

    // Build full markdown
    let full_content = if frontmatter.trim().is_empty() {
        content.to_string()
    } else {
        format!("---\n{}---\n\n{}", frontmatter, content)
    };

    // Write file directly (synchronous) to avoid race with graph rebuild
    if let Some(parent) = full_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| ToolError::internal(format!("Failed to create directory: {}", e)))?;
    }
    std::fs::write(&full_path, full_content.as_bytes())
        .map_err(|e| ToolError::internal(format!("Failed to write page: {}", e)))?;

    let meta = parse_wiki_page(&full_path, &full_content);
    engine.stale_flag.store(true, Ordering::Release);

    Ok(meta.id)
}

/// Get a page by its wiki ID (e.g., "wiki:concepts:auth")
pub fn get_page(engine: &Arc<EngineState>, id: &str) -> ToolResult<WikiPageContent> {
    // Look up in the graph snapshot
    let snapshot = engine.graph.load();
    let id_index = &snapshot.1;

    let _node_idx = id_index
        .get(id)
        .ok_or_else(|| ToolError::not_found("page", id))?;

    // Read from disk
    let root = Path::new(".");
    let file_path = resolve_id_to_path(root, id)?;
    let content = std::fs::read_to_string(&file_path).map_err(|e| {
        ToolError::internal(format!("Failed to read {}: {}", file_path.display(), e))
    })?;

    let sections = crate::parser::split_sections(&content);

    Ok(WikiPageContent {
        raw: content,
        sections: sections
            .into_iter()
            .map(|(header, body)| {
                let section_id = format!("{}#{}", id, header.to_lowercase().replace(' ', "-"));
                crate::engine::SectionDoc {
                    section_id,
                    page_id: id.to_string(),
                    header,
                    body,
                }
            })
            .collect(),
    })
}

/// Get a page's raw markdown content by its wiki ID (by reading from disk).
/// This bypasses the graph snapshot and reads the file directly.
pub fn get_page_raw(engine: &EngineState, id: &str) -> ToolResult<String> {
    let snapshot = engine.graph.load();
    let index = &snapshot.1;
    let node_idx = index
        .get(id)
        .ok_or_else(|| ToolError::not_found("page", id))?;
    let meta = &snapshot.0[*node_idx];
    let file_path = &meta.path;

    std::fs::read_to_string(file_path).map_err(|e| {
        ToolError::internal(format!("Failed to read {}: {}", file_path.display(), e))
    })
}

/// List all page IDs and titles, optionally filtered by page type.
pub fn list_pages(engine: &Arc<EngineState>, page_type_filter: Option<&PageType>) -> ToolResult<Vec<serde_json::Value>> {
    let snapshot = engine.graph.load();
    let graph = &snapshot.0;

    let pages: Vec<serde_json::Value> = graph
        .node_indices()
        .filter_map(|idx| {
            let meta = &graph[idx];
            if let Some(pt) = page_type_filter {
                if meta.page_type != *pt {
                    return None;
                }
            }
            Some(serde_json::json!({
                "id": meta.id,
                "title": meta.title,
                "type": meta.page_type.as_str(),
                "status": meta.status.as_str(),
            }))
        })
        .collect();

    Ok(pages)
}

/// Delete a wiki page by its ID. Removes the file and marks the engine stale.
pub fn delete_page(engine: &Arc<EngineState>, id: &str) -> ToolResult<()> {
    let snapshot = engine.graph.load();
    let index = &snapshot.1;
    let node_idx = index
        .get(id)
        .ok_or_else(|| ToolError::not_found("page", id))?;
    let meta = &snapshot.0[*node_idx];
    let file_path = &meta.path;

    if file_path.exists() {
        std::fs::remove_file(file_path).map_err(|e| {
            ToolError::internal(format!(
                "Failed to delete {}: {}",
                file_path.display(),
                e
            ))
        })?;
    }

    engine
        .stale_flag
        .store(true, std::sync::atomic::Ordering::Release);

    Ok(())
}

#[derive(Default)]
pub struct PageUpdateParams {
    pub title: Option<String>,
    pub content: Option<String>,
    pub status: Option<String>,
    pub priority: Option<String>,
    pub assignee: Option<String>,
    pub tags: Option<Vec<String>>,
    pub relates_to: Option<Vec<serde_json::Value>>,
    pub remove_relates_to: Option<String>,
    pub acceptance_criteria: Option<Vec<AcceptanceCriterion>>,
    pub implementation_plan: Option<String>,
    pub implementation_notes: Option<String>,
    pub append_notes: Option<String>,
    pub r#type: Option<String>,
    pub checked_ac: Option<Vec<u64>>,
    pub unchecked_ac: Option<Vec<u64>>,
    pub time_started: Option<String>,
    pub time_spent: Option<String>,
}

/// Update an existing wiki page — merge new frontmatter fields
pub fn update_page(
    engine: &Arc<EngineState>,
    id: &str,
    updates: &PageUpdateParams,
) -> ToolResult<()> {
    // Find the page path from graph
    let snapshot = engine.graph.load();
    let index = &snapshot.1;
    let node_idx = index
        .get(id)
        .ok_or_else(|| ToolError::not_found("page", id))?;
    let meta = &snapshot.0[*node_idx];

    let file_path = &meta.path;
    if !file_path.exists() {
        return Err(ToolError::not_found("page", id));
    }

    let content = std::fs::read_to_string(file_path).map_err(|e| {
        ToolError::internal(format!("Failed to read {}: {}", file_path.display(), e))
    })?;

    // Parse existing frontmatter
    let (existing_fm, body) = crate::parser::extract_frontmatter(&content);

    // Build updated frontmatter YAML from shared serializer
    let mut new_fm = existing_fm
        .as_ref()
        .map(crate::parser::frontmatter_to_yaml)
        .unwrap_or_default();

    // Validate state transition for task pages using file's current status (not snapshot)
    if let Some(status) = updates.status.as_deref() {
        if meta.page_type == PageType::Task {
            let new_status = crate::parser::parse_page_status(status);
            // Use the file's frontmatter status (more current than the graph snapshot)
            let file_status_str = existing_fm.as_ref().and_then(|fm| fm.status.as_deref());
            let current_status = file_status_str
                .map(crate::parser::parse_page_status)
                .unwrap_or_else(|| meta.status.clone());
            if let Err(msg) = current_status.can_transition_to(&new_status) {
                return Err(ToolError::internal(msg));
            }
        }
        new_fm = set_yaml_field(&new_fm, "status", status);
    }

    // Handle priority override
    if let Some(priority) = updates.priority.as_deref() {
        new_fm = set_yaml_field(&new_fm, "priority", priority);
    }

    // Handle assignee override
    if let Some(assignee) = updates.assignee.as_deref() {
        new_fm = set_yaml_field(&new_fm, "assignee", assignee);
    }

    // Handle tags replacement
    if let Some(ref tag_list) = updates.tags {
        new_fm = remove_yaml_block(&new_fm, "tags");
        if !tag_list.is_empty() {
            new_fm.push_str(&format!("tags: [{}]\n", tag_list.join(", ")));
        }
    }

    // Handle acceptance_criteria replacement (expects array of {text, checked} objects)
    if let Some(ref ac_list) = updates.acceptance_criteria {
        new_fm = remove_yaml_block(&new_fm, "acceptance_criteria");
        if !ac_list.is_empty() {
            new_fm.push_str("acceptance_criteria:\n");
            for ac in ac_list {
                new_fm.push_str(&format!("  - {{text: \"{}\", checked: {}}}\n", ac.text, ac.checked));
            }
        }
    }

    // Handle implementation_plan set/replace
    if let Some(ref plan) = updates.implementation_plan {
        new_fm = set_yaml_field(&new_fm, "implementation_plan", plan);
    }

    // Handle implementation_notes set/replace
    if let Some(ref notes) = updates.implementation_notes {
        new_fm = set_yaml_field(&new_fm, "implementation_notes", notes);
    }

    // Handle implementation_notes append
    if let Some(ref append) = updates.append_notes {
        // Read existing implementation_notes from the raw YAML string
        let existing = extract_yaml_string_value(&new_fm, "implementation_notes");
        let merged = if existing.is_empty() {
            append.to_string()
        } else {
            format!("{}\n{}", existing, append)
        };
        new_fm = set_yaml_field(&new_fm, "implementation_notes", &merged);
    }

    // Handle content (body) override
    let final_body = if let Some(new_content) = updates.content.as_deref() {
        new_content
    } else {
        body
    };

    // Handle relates_to: replace all entries
    if let Some(ref rel_list) = updates.relates_to {
        new_fm = remove_yaml_block(&new_fm, "relates_to");
        if !rel_list.is_empty() {
            new_fm.push_str("relates_to:\n");
            for rel in rel_list {
                let edge_type = rel
                    .get("type")
                    .and_then(|v| v.as_str())
                    .unwrap_or("relates_to");
                let target = rel.get("target").and_then(|v| v.as_str()).unwrap_or("");
                new_fm.push_str(&format!(
                    "  - {{type: {}, target: {}}}\n",
                    edge_type, target
                ));
            }
        }
    }

    // Handle remove_relates_to: remove entries matching a target
    if let Some(remove_target) = updates.remove_relates_to.as_deref() {
        // Collect existing relates_to lines that don't match the target
        let mut kept: Vec<String> = Vec::new();
        for line in new_fm.lines() {
            if line.trim().starts_with("- {") && line.contains(remove_target) {
                continue; // skip removed target
            }
            kept.push(line.to_string());
        }
        // Rebuild frontmatter without the removed entries
        new_fm = remove_yaml_block(&new_fm, "relates_to");
        let has_relates = kept.iter().any(|l| l.trim().starts_with("- {"));
        if has_relates {
            new_fm.push_str("relates_to:\n");
            for k in kept {
                if k.trim().starts_with("- {") {
                    new_fm.push_str(&format!("  {}\n", k.trim_start()));
                }
            }
        }
    }

    // Handle checked_ac / unchecked_ac
    if let Some(ref check_list) = updates.checked_ac {
        for &idx in check_list.iter() {
            new_fm = ac_set_checked(&new_fm, idx as usize, true);
        }
    }
    if let Some(ref uncheck_list) = updates.unchecked_ac {
        for &idx in uncheck_list.iter() {
            new_fm = ac_set_checked(&new_fm, idx as usize, false);
        }
    }

    let full = format!("---\n{}---\n\n{}", new_fm, final_body);
    std::fs::write(file_path.clone(), full.into_bytes())
        .map_err(|e| ToolError::internal(format!("Failed to write page update: {}", e)))?;

    engine.stale_flag.store(true, Ordering::Release);
    Ok(())
}

fn parse_yaml_mut<F>(yaml: &str, f: F) -> String
where
    F: FnOnce(&mut serde_yaml::Mapping),
{
    let mut value: serde_yaml::Value = serde_yaml::from_str(yaml)
        .unwrap_or(serde_yaml::Value::Null);
    // Handle Null by replacing with empty Mapping
    if value.is_null() {
        value = serde_yaml::Value::Mapping(serde_yaml::Mapping::new());
    }
    if let serde_yaml::Value::Mapping(ref mut map) = value {
        f(map);
    }
    serde_yaml::to_string(&value).unwrap_or_else(|_| yaml.to_string())
}

/// Extract a string value from a YAML string by key. Returns empty string if missing.
fn extract_yaml_string_value(yaml: &str, key: &str) -> String {
    let value: serde_yaml::Value = serde_yaml::from_str(yaml).unwrap_or(serde_yaml::Value::Null);
    match value {
        serde_yaml::Value::Mapping(ref map) => {
            let k = serde_yaml::Value::String(key.to_string());
            map.get(&k)
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .unwrap_or_default()
        }
        _ => String::new(),
    }
}

fn set_yaml_field(yaml: &str, key: &str, value: &str) -> String {
    parse_yaml_mut(yaml, |map| {
        map.insert(
            serde_yaml::Value::String(key.to_string()),
            serde_yaml::Value::String(value.to_string()),
        );
    })
}

fn ac_set_checked(yaml: &str, index: usize, checked: bool) -> String {
    parse_yaml_mut(yaml, |map| {
        if let Some(serde_yaml::Value::Sequence(ref mut items)) =
            map.get_mut(serde_yaml::Value::String("acceptance_criteria".to_string()))
        {
            if index > 0 && index <= items.len() {
                if let serde_yaml::Value::Mapping(ref mut ac_map) = items[index - 1] {
                    ac_map.insert(
                        serde_yaml::Value::String("checked".to_string()),
                        serde_yaml::Value::Bool(checked),
                    );
                }
            }
        }
    })
}

/// Remove a YAML multi-line block (e.g., relates_to or acceptance_criteria) from frontmatter.
fn remove_yaml_block(yaml: &str, key: &str) -> String {
    parse_yaml_mut(yaml, |map| {
        map.remove(serde_yaml::Value::String(key.to_string()));
    })
}

fn resolve_page_path(_project_name: &str, path: &str) -> ToolResult<PathBuf> {
    // If path has .md, use as-is relative to wiki dir
    let wiki_dir = Path::new(".wm").join("wiki");
    let file_path = if path.ends_with(".md") {
        wiki_dir.join(path.trim_start_matches("wiki/"))
    } else {
        // Generate path from ID: "wiki:concepts:auth" → "wiki/concepts/auth.md"
        let path_part = path.replace(':', "/");
        wiki_dir.join(format!("{}.md", path_part.trim_start_matches("wiki/")))
    };

    // Ensure it's within the wiki directory
    if !file_path.starts_with(&wiki_dir) {
        return Err(ToolError::required_field("path"));
    }

    Ok(file_path)
}

fn resolve_id_to_path(project_root: &Path, id: &str) -> ToolResult<PathBuf> {
    let path_part = id.replace(':', "/");
    // Strip leading "wiki/" since it's added by path_to_id but files are relative to wiki dir
    let path_part = path_part.strip_prefix("wiki/").unwrap_or(&path_part);
    let file_path = project_root
        .join(".wm")
        .join("wiki")
        .join(format!("{}.md", path_part));
    if file_path.exists() {
        Ok(file_path)
    } else {
        Err(ToolError::not_found("page", id))
    }
}

// ─── Migration: JSON Memory Files → Wiki Pages ───────────────

/// Migrate old-style `.wm/memory/*.json` files to `.wm/wiki/memory/*.md` wiki pages.
///
/// Each JSON file is read as a `MemoryEntry`, converted to a markdown page with
/// YAML frontmatter, and written to the wiki memory directory. The JSON file is
/// then removed on success.
///
/// Returns the number of migrated entries.
pub fn migrate_old_memory_json(engine: &Arc<EngineState>) -> ToolResult<usize> {
    let root = engine
        .project_root
        .read()
        .map(|r| r.clone())
        .unwrap_or_else(|_| PathBuf::from("."));
    let old_dir = root.join(".wm").join("memory");

    if !old_dir.exists() {
        return Ok(0);
    }

    let mut migrated = 0usize;

    // Collect JSON files
    let entries = match std::fs::read_dir(&old_dir) {
        Ok(e) => e,
        Err(_) => return Ok(0),
    };

    for entry in entries {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("json") {
            continue;
        }

        // Read and parse the old JSON file
        let content = match std::fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let mem: crate::engine::MemoryEntry = match serde_json::from_str(&content) {
            Ok(m) => m,
            Err(_) => continue,
        };

        // Build frontmatter
        let tags_str = if mem.tags.is_empty() {
            String::new()
        } else {
            format!("tags: [{}]\n", mem.tags.join(", "))
        };
        let status_str = mem.status.as_ref().map(|s| format!("status: {:?}\n", s)).unwrap_or_default();
        let frontmatter = format!(
            "title: {}\ntype: memory\n{}created_at: \"{}\"\nupdated_at: \"{}\"\n{}",
            mem.title, tags_str, mem.created_at, mem.updated_at, status_str
        );

        // Write the wiki page
        let slug = mem.id;
        let rel_path = format!("memory/{}", slug);
        let _ = crate::page::create_page(engine, &rel_path, &frontmatter, &mem.content);

        // Remove old JSON file
        let _ = std::fs::remove_file(&path);

        migrated += 1;
    }

    // Remove old memory directory if empty
    if migrated > 0 {
        let _ = std::fs::remove_dir(&old_dir);
    }

    tracing::info!("Migrated {} memory entries from JSON to wiki pages", migrated);
    Ok(migrated)
}

// ─── Orphan Timer Recovery (moved from source.rs) ─────────────

/// Check for orphan timers on startup — any time_started > 24h
/// Auto-closes by setting status to done and computing time_spent.
/// Operates on task pages (not source entries), so it lives in page.rs.
pub fn recover_orphan_timers(engine: &Arc<EngineState>) -> ToolResult<usize> {
    use chrono::Utc;
    let mut recovered = 0;
    let snapshot = engine.graph.load();
    let graph = &snapshot.0;
    let index = &snapshot.1;

    for (page_id, node_idx) in index {
        let meta = &graph[*node_idx];
        if meta.page_type != PageType::Task {
            continue;
        }

        let path = resolve_simple_page_path(page_id);
        let content = match std::fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let (fm, body) = parser::extract_frontmatter(&content);
        let fm = match fm {
            Some(f) => f,
            None => continue,
        };

        // Check if time_started is set and status is still in-progress/todo
        let time_started = match fm.time_started {
            Some(ref t) => t.clone(),
            None => continue,
        };

        // Parse timestamp (ISO 8601)
        let started_at = match chrono::DateTime::parse_from_rfc3339(&time_started) {
            Ok(t) => t,
            Err(_) => continue,
        };

        // Check if more than 24h have elapsed
        let elapsed = Utc::now().signed_duration_since(started_at);
        if elapsed.num_hours() < 24 {
            continue;
        }

        // Auto-close: compute time_spent, set status=done
        let hours = elapsed.num_hours();
        let minutes = elapsed.num_minutes() % 60;
        let time_spent = format!("{}h {}m", hours, minutes);

        // Build updated frontmatter from shared serializer, then override
        let mut new_fm = crate::parser::frontmatter_to_yaml(&fm);
        new_fm.push_str("status: done\n");
        new_fm.push_str(&format!("time_started: {}\n", time_started));
        new_fm.push_str(&format!("time_spent: {}\n", time_spent));

        let full = format!("---\n{}---\n\n{}", new_fm, body);
        if engine
            .write_channel
            .write(path.clone(), full.into_bytes())
            .is_ok()
        {
            tracing::info!(
                "Recovered orphan timer: {} ({} elapsed)",
                page_id,
                time_spent
            );
            recovered += 1;

            engine.emit_audit(
                "page.recover",
                "auto-close",
                "ok",
                0,
                None,
                vec![page_id.clone()],
            );
        }
    }

    Ok(recovered)
}

fn resolve_simple_page_path(id: &str) -> PathBuf {
    let path_part = id.replace(':', "/");
    PathBuf::from(".wm")
        .join("wiki")
        .join(format!("{}.md", path_part))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_yaml_mut_set_field() {
        let yaml = "title: Test\ntype: task\n";
        let result = parse_yaml_mut(yaml, |map| {
            map.insert(
                serde_yaml::Value::String("status".to_string()),
                serde_yaml::Value::String("done".to_string()),
            );
        });
        assert!(result.contains("status: done"), "Result should contain new field: {}", result);
        assert!(result.contains("title: Test"), "Result should preserve existing field: {}", result);
    }

    #[test]
    fn test_parse_yaml_mut_empty() {
        let result = parse_yaml_mut("", |map| {
            map.insert(
                serde_yaml::Value::String("key".to_string()),
                serde_yaml::Value::String("value".to_string()),
            );
        });
        assert!(result.contains("key: value"), "Empty YAML should produce new key: {}", result);
    }

    #[test]
    fn test_set_yaml_field() {
        let yaml = "title: Test\n";
        let result = set_yaml_field(yaml, "status", "done");
        assert!(result.contains("status: done"), "Result: {}", result);
        assert!(result.contains("title: Test"), "Result: {}", result);
    }

    #[test]
    fn test_ac_set_checked() {
        let yaml = r#"acceptance_criteria:
  - {text: "works", checked: false}
  - {text: "tested", checked: true}
"#;
        let result = ac_set_checked(yaml, 1, true);
        assert!(result.contains("checked: true"), "First AC should be checked: {}", result);
        assert!(result.contains("works"), "Text should be preserved: {}", result);

        let result2 = ac_set_checked(yaml, 2, false);
        assert!(result2.contains("checked: false"), "Second AC should be unchecked: {}", result2);
    }

    #[test]
    fn test_remove_yaml_block() {
        let yaml = r#"title: Test
relates_to:
  - {type: extends, target: other}
tags: [a, b]
"#;
        let result = remove_yaml_block(yaml, "relates_to");
        assert!(!result.contains("relates_to"), "Block should be removed: {}", result);
        assert!(result.contains("title: Test"), "Other fields preserved: {}", result);
        assert!(result.contains("tags:"), "Other fields preserved: {}", result);
    }

    #[test]
    fn test_resolve_page_path_prevents_traversal() {
        // Verify that path traversal attempts produce paths within the wiki directory
        let result = crate::page::resolve_page_path("test-proj", "../../etc/passwd");
        match result {
            Ok(path) => {
                // Even if it resolves, path must still be within wiki dir
                assert!(path.starts_with(".wm\\wiki") || path.starts_with(".wm/wiki"),
                    "path should stay within wiki dir: {:?}", path);
            }
            Err(_) => {} // rejected = acceptable
        }

        let result2 = crate::page::resolve_page_path("test-proj", "/etc/passwd");
        match result2 {
            Ok(path) => {
                // Must stay within wiki dir
                assert!(path.starts_with(".wm\\wiki") || path.starts_with(".wm/wiki"),
                    "path should stay within wiki dir: {:?}", path);
            }
            Err(_) => {} // rejected = acceptable
        }
    }

    #[test]
    fn test_migrate_no_memory_dir() {
        let tmp = std::env::temp_dir().join("wm-test-no-mem");
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(tmp.join(".wm").join("wiki")).unwrap();

        // Create an EngineState with project_root pointing at the temp dir
        let rt = tokio::runtime::Runtime::new().unwrap();
        let _guard = rt.enter();
        let (engine, _rx) = crate::engine::EngineState::new(crate::config::ProjectConfig::default());
        {
            let mut root = engine.project_root.write().unwrap();
            *root = tmp.clone();
        }
        let engine = Arc::new(engine);

        // Should not panic when memory dir doesn't exist
        let result = crate::page::migrate_old_memory_json(&engine);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_migrate_invalid_json() {
        let tmp = std::env::temp_dir().join("wm-test-bad-json");
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(tmp.join(".wm").join("memory")).unwrap();
        std::fs::write(
            tmp.join(".wm").join("memory").join("bad.json"),
            "not valid json",
        )
        .unwrap();
        std::fs::create_dir_all(tmp.join(".wm").join("wiki")).unwrap();

        // Create an EngineState with project_root pointing at the temp dir
        let rt = tokio::runtime::Runtime::new().unwrap();
        let _guard = rt.enter();
        let (engine, _rx) = crate::engine::EngineState::new(crate::config::ProjectConfig::default());
        {
            let mut root = engine.project_root.write().unwrap();
            *root = tmp.clone();
        }
        let engine = Arc::new(engine);

        // Should skip invalid file without panicking
        let result = crate::page::migrate_old_memory_json(&engine);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0, "should migrate 0 files when all are invalid");

        let _ = std::fs::remove_dir_all(&tmp);
    }
}
