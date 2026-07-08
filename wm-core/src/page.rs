use std::path::{Path, PathBuf};
use std::sync::atomic::Ordering;
use std::sync::Arc;

use crate::engine::{EngineState, PageType, WikiPageContent};
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

/// List all page IDs and titles
pub fn list_pages(engine: &Arc<EngineState>) -> ToolResult<Vec<serde_json::Value>> {
    let snapshot = engine.graph.load();
    let graph = &snapshot.0;

    let pages: Vec<serde_json::Value> = graph
        .node_indices()
        .map(|idx| {
            let meta = &graph[idx];
            serde_json::json!({
                "id": meta.id,
                "title": meta.title,
                "type": format!("{:?}", meta.page_type).to_lowercase(),
                "status": format!("{:?}", meta.status).to_lowercase(),
            })
        })
        .collect();

    Ok(pages)
}

/// Update an existing wiki page — merge new frontmatter fields
pub fn update_page(
    engine: &Arc<EngineState>,
    id: &str,
    updates: &serde_json::Value,
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

    // Build updated frontmatter YAML
    let mut new_fm = String::new();

    if let Some(ref fm) = existing_fm {
        if let Some(ref title) = fm.title {
            new_fm.push_str(&format!("title: {}\n", title));
        }
        if let Some(ref pt) = fm.page_type {
            new_fm.push_str(&format!("type: {}\n", pt));
        }
        if !fm.tags.is_empty() {
            new_fm.push_str(&format!("tags: [{}]\n", fm.tags.join(", ")));
        }
        if let Some(ref s) = fm.status {
            new_fm.push_str(&format!("status: {}\n", s));
        }
        if let Some(ref p) = fm.priority {
            new_fm.push_str(&format!("priority: {}\n", p));
        }
        if let Some(ref c) = fm.confidence {
            new_fm.push_str(&format!("confidence: {}\n", c));
        }
        if let Some(ref a) = fm.assignee {
            new_fm.push_str(&format!("assignee: {}\n", a));
        }
        if !fm.aliases.is_empty() {
            new_fm.push_str(&format!("aliases: [{}]\n", fm.aliases.join(", ")));
        }
        if !fm.sources.is_empty() {
            new_fm.push_str(&format!("sources: [{}]\n", fm.sources.join(", ")));
        }
        if let Some(ref s) = fm.superseded_by {
            new_fm.push_str(&format!("superseded_by: {}\n", s));
        }
        if let Some(ref v) = fm.version {
            new_fm.push_str(&format!("version: {}\n", v));
        }
        if let Some(ref est) = fm.estimate {
            new_fm.push_str(&format!("estimate: {}\n", est));
        }
        if !fm.prerequisites.is_empty() {
            new_fm.push_str(&format!(
                "prerequisites: [{}]\n",
                fm.prerequisites.join(", ")
            ));
        }
        if let Some(ref d) = fm.difficulty {
            new_fm.push_str(&format!("difficulty: {}\n", d));
        }
        if let Some(ref src) = fm.source_url {
            new_fm.push_str(&format!("source_url: {}\n", src));
        }
        if !fm.stakeholders.is_empty() {
            new_fm.push_str(&format!("stakeholders: [{}]\n", fm.stakeholders.join(", ")));
        }
        if let Some(ref t) = fm.time_started {
            new_fm.push_str(&format!("time_started: {}\n", t));
        }
        if let Some(ref t) = fm.time_spent {
            new_fm.push_str(&format!("time_spent: {}\n", t));
        }
        // Per-type structured fields
        if !fm.functional_requirements.is_empty() {
            new_fm.push_str("functional_requirements:\n");
            for fr in &fm.functional_requirements {
                new_fm.push_str(&format!(
                    "  - {{id: {}, description: \"{}\"}}\n",
                    fr.id, fr.description
                ));
            }
        }
        if !fm.non_functional_requirements.is_empty() {
            new_fm.push_str("non_functional_requirements:\n");
            for nfr in &fm.non_functional_requirements {
                new_fm.push_str(&format!(
                    "  - {{id: {}, description: \"{}\"}}\n",
                    nfr.id, nfr.description
                ));
            }
        }
        if !fm.general_goals.is_empty() {
            new_fm.push_str("general_goals:\n");
            for g in &fm.general_goals {
                new_fm.push_str(&format!("  - {{description: \"{}\"}}\n", g.description));
            }
        }
        if let Some(ref dec) = fm.decision {
            new_fm.push_str("decision:\n");
            new_fm.push_str(&format!("  context: \"{}\"\n", dec.context));
            if !dec.options.is_empty() {
                new_fm.push_str(&format!("  options: [{}]\n", dec.options.join(", ")));
            }
            new_fm.push_str(&format!("  rationale: \"{}\"\n", dec.rationale));
            new_fm.push_str(&format!("  outcome: \"{}\"\n", dec.outcome));
        }
        if let Some(ref pat) = fm.pattern {
            new_fm.push_str("pattern:\n");
            new_fm.push_str(&format!("  when_to_use: \"{}\"\n", pat.when_to_use));
            new_fm.push_str(&format!("  example: \"{}\"\n", pat.example));
        }
        if !fm.relates_to.is_empty() {
            new_fm.push_str("relates_to:\n");
            for r in &fm.relates_to {
                new_fm.push_str(&format!(
                    "  - {{type: {}, target: {}}}\n",
                    r.edge_type, r.target
                ));
            }
        }
        if !fm.acceptance_criteria.is_empty() {
            new_fm.push_str("acceptance_criteria:\n");
            for ac in &fm.acceptance_criteria {
                new_fm.push_str(&format!(
                    "  - {{text: \"{}\", checked: {}}}\n",
                    ac.text, ac.checked
                ));
            }
        }
    }

    // Override with update fields
    if let Some(title) = updates.get("title").and_then(|v| v.as_str()) {
        new_fm = set_yaml_field(&new_fm, "title", title);
    }
    if let Some(status) = updates.get("status").and_then(|v| v.as_str()) {
        new_fm = set_yaml_field(&new_fm, "status", status);
    }

    // Handle priority override
    if let Some(priority) = updates.get("priority").and_then(|v| v.as_str()) {
        new_fm = set_yaml_field(&new_fm, "priority", priority);
    }

    // Handle assignee override
    if let Some(assignee) = updates.get("assignee").and_then(|v| v.as_str()) {
        new_fm = set_yaml_field(&new_fm, "assignee", assignee);
    }

    // Handle tags replacement
    if updates.get("tags").and_then(|v| v.as_array()).is_some() {
        new_fm = remove_yaml_block(&new_fm, "tags");
        if let Some(tag_list) = updates.get("tags").and_then(|v| v.as_array()) {
            if !tag_list.is_empty() {
                let tags: Vec<String> = tag_list
                    .iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect();
                new_fm.push_str(&format!("tags: [{}]\n", tags.join(", ")));
            }
        }
    }

    // Handle acceptance_criteria replacement (expects array of {text, checked} objects)
    if updates.get("acceptance_criteria").and_then(|v| v.as_array()).is_some() {
        new_fm = remove_yaml_block(&new_fm, "acceptance_criteria");
        if let Some(ac_list) = updates.get("acceptance_criteria").and_then(|v| v.as_array()) {
            if !ac_list.is_empty() {
                new_fm.push_str("acceptance_criteria:\n");
                for ac in ac_list {
                    let text = ac.get("text").and_then(|v| v.as_str()).unwrap_or("");
                    let checked = ac.get("checked").and_then(|v| v.as_bool()).unwrap_or(false);
                    new_fm.push_str(&format!("  - {{text: \"{}\", checked: {}}}\n", text, checked));
                }
            }
        }
    }

    // Handle content (body) override
    let final_body = if let Some(new_content) = updates.get("content").and_then(|v| v.as_str()) {
        new_content
    } else {
        body
    };

    // Handle relates_to: replace all entries
    if let Some(rel_list) = updates.get("relates_to").and_then(|v| v.as_array()) {
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
    if let Some(remove_target) = updates.get("remove_relates_to").and_then(|v| v.as_str()) {
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
    if let Some(check_list) = updates.get("checked_ac").and_then(|v| v.as_array()) {
        for idx in check_list.iter().filter_map(|v| v.as_u64()) {
            new_fm = ac_set_checked(&new_fm, idx as usize, true);
        }
    }
    if let Some(uncheck_list) = updates.get("unchecked_ac").and_then(|v| v.as_array()) {
        for idx in uncheck_list.iter().filter_map(|v| v.as_u64()) {
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
            map.get_mut(&serde_yaml::Value::String("acceptance_criteria".to_string()))
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
        map.remove(&serde_yaml::Value::String(key.to_string()));
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

        // Build updated frontmatter — preserve all existing fields, override status
        let mut new_fm = String::new();
        if let Some(ref title) = fm.title {
            new_fm.push_str(&format!("title: {}\n", title));
        }
        if let Some(ref pt) = fm.page_type {
            new_fm.push_str(&format!("type: {}\n", pt));
        }
        if !fm.tags.is_empty() {
            new_fm.push_str(&format!("tags: [{}]\n", fm.tags.join(", ")));
        }
        if let Some(ref p) = fm.priority {
            new_fm.push_str(&format!("priority: {}\n", p));
        }
        if let Some(ref c) = fm.confidence {
            new_fm.push_str(&format!("confidence: {}\n", c));
        }
        if let Some(ref a) = fm.assignee {
            new_fm.push_str(&format!("assignee: {}\n", a));
        }
        if let Some(ref s) = fm.superseded_by {
            new_fm.push_str(&format!("superseded_by: {}\n", s));
        }
        if let Some(ref v) = fm.version {
            new_fm.push_str(&format!("version: {}\n", v));
        }
        if let Some(ref est) = fm.estimate {
            new_fm.push_str(&format!("estimate: {}\n", est));
        }
        if let Some(ref d) = fm.difficulty {
            new_fm.push_str(&format!("difficulty: {}\n", d));
        }
        if !fm.prerequisites.is_empty() {
            new_fm.push_str(&format!(
                "prerequisites: [{}]\n",
                fm.prerequisites.join(", ")
            ));
        }
        if let Some(ref src) = fm.source_url {
            new_fm.push_str(&format!("source_url: {}\n", src));
        }
        if !fm.relates_to.is_empty() {
            new_fm.push_str("relates_to:\n");
            for r in &fm.relates_to {
                new_fm.push_str(&format!(
                    "  - {{type: {}, target: {}}}\n",
                    r.edge_type, r.target
                ));
            }
        }
        if !fm.acceptance_criteria.is_empty() {
            new_fm.push_str("acceptance_criteria:\n");
            for ac in &fm.acceptance_criteria {
                new_fm.push_str(&format!(
                    "  - {{text: \"{}\", checked: {}}}\n",
                    ac.text, ac.checked
                ));
            }
        }
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
}
