use std::sync::Arc;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::engine::EngineState;
use wm_error::ToolError;
use crate::mcp::transport::ToolRegistry;

use std::path::PathBuf;

/// Resolve the wiki documents directory (.wm/wiki/) from a project root.
fn wiki_docs_dir(root: &std::path::Path) -> PathBuf {
    root.join(".wm").join("wiki")
}

/// Append `.md` to a path if it doesn't already end with `.md`.
fn ensure_md_ext(path: &str) -> String {
    if path.ends_with(".md") {
        path.to_string()
    } else {
        format!("{}.md", path)
    }
}

// ─── Action enum ────────────────────────────────────────────

#[derive(Deserialize, JsonSchema)]
#[serde(tag = "action")]
enum WmDocAction {
    #[schemars(description = "List documents in the wiki (.wm/wiki/)")]
    List {
        r#type: Option<String>,
    },
    #[schemars(description = "Read a doc from .wm/wiki/ by path")]
    Get { path: String },
    #[schemars(description = "Create a new doc in .wm/wiki/")]
    Create {
        path: String,
        title: String,
        content: Option<String>,
        r#type: Option<String>,
        tags: Option<Vec<String>>,
    },
    #[schemars(description = "Update an existing doc")]
    Update {
        path: String,
        title: Option<String>,
        content: Option<String>,
    },
    #[schemars(description = "Delete a doc")]
    Delete { path: String },
}

// ─── Output types ───────────────────────────────────────────

#[derive(Serialize)]
struct WmDocGetOutput {
    path: String,
    title: String,
    content: String,
    body: String,
    frontmatter: serde_json::Map<String, serde_json::Value>,
    tags: Vec<String>,
}

#[derive(Serialize)]
struct WmDocCreateOutput {
    path: String,
    title: String,
    tags: Vec<String>,
    status: String,
}

#[derive(Serialize)]
struct WmDocUpdateOutput {
    path: String,
    status: String,
}

#[derive(Serialize)]
struct WmDocDeleteOutput {
    path: String,
    status: String,
}

/// Register the single wm_doc tool
pub fn register(registry: &mut ToolRegistry, engine: Arc<EngineState>) {
    registry.register_typed(
        "wm_doc",
        "Doc CRUD operations: list, get, create, update, delete",
        move |input: WmDocAction| -> Result<serde_json::Value, ToolError> {
            match input {
                WmDocAction::List { r#type } => {
                    let folder = r#type;

                    let root = engine
                        .project_root
                        .read()
                        .map_err(|_| ToolError::lock_poisoned("project_root"))?
                        .clone();

                    let wiki_dir = wiki_docs_dir(&root);
                    if !wiki_dir.exists() || !wiki_dir.is_dir() {
                        return Ok(serde_json::json!({
                            "docs": [],
                            "total": 0,
                            "path": wiki_dir.to_string_lossy(),
                            "note": ".wm/wiki/ not found"
                        }));
                    }

                    let walk_dir = match &folder {
                        Some(f) => wiki_dir.join(f),
                        None => wiki_dir.clone(),
                    };

                    if !walk_dir.exists() || !walk_dir.is_dir() {
                        return Ok(serde_json::json!({
                            "docs": [],
                            "total": 0,
                            "path": walk_dir.to_string_lossy(),
                            "note": "folder not found"
                        }));
                    }

                    let mut docs: Vec<serde_json::Value> = Vec::new();

                    let entries = match std::fs::read_dir(&walk_dir) {
                        Ok(entries) => entries,
                        Err(e) => {
                            return Err(ToolError::io_error(
                                "read_dir",
                                walk_dir.to_string_lossy(),
                                e,
                            ))
                        }
                    };

                    for entry in entries {
                        let entry = match entry {
                            Ok(e) => e,
                            Err(_) => continue,
                        };
                        let path = entry.path();
                        if path.extension().and_then(|s| s.to_str()) != Some("md") {
                            continue;
                        }
                        if !path.is_file() {
                            continue;
                        }

                        // Read file and parse YAML frontmatter
                        let content =
                            std::fs::read_to_string(&path).unwrap_or_default();
                        let (frontmatter, _body) = parse_frontmatter(&content);

                        // Extract title from frontmatter or fall back to filename stem
                        let title = frontmatter
                            .get("title")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string())
                            .filter(|s| !s.is_empty())
                            .unwrap_or_else(|| {
                                path.file_stem()
                                    .and_then(|s| s.to_str())
                                    .unwrap_or("")
                                    .to_string()
                            });

                        // Extract tags (array)
                        let tags: Vec<String> = frontmatter
                            .get("tags")
                            .and_then(|v| v.as_array())
                            .map(|arr| {
                                arr.iter()
                                    .filter_map(|v| v.as_str().map(String::from))
                                    .collect()
                            })
                            .unwrap_or_default();

                        // Extract scalar fields (fall back to empty string)
                        let description = frontmatter
                            .get("description")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string();
                        let created_at = frontmatter
                            .get("createdAt")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string();
                        let updated_at = frontmatter
                            .get("updatedAt")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string();

                        // Determine folder relative to .wm/wiki/
                        let doc_folder = path
                            .parent()
                            .and_then(|p| p.strip_prefix(&wiki_dir).ok())
                            .map(|p| p.to_string_lossy().to_string())
                            .filter(|s| !s.is_empty())
                            .unwrap_or_default();

                        let doc_path = path
                            .strip_prefix(&root)
                            .unwrap_or(&path)
                            .to_string_lossy()
                            .to_string();

                        docs.push(serde_json::json!({
                            "path": doc_path,
                            "title": title,
                            "folder": doc_folder,
                            "tags": tags,
                            "description": description,
                            "createdAt": created_at,
                            "updatedAt": updated_at,
                        }));
                    }

                    // Sort by path for stable ordering
                    docs.sort_by(|a, b| {
                        a.get("path")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .cmp(b.get("path").and_then(|v| v.as_str()).unwrap_or(""))
                    });

                    Ok(serde_json::json!({
                        "docs": docs,
                        "total": docs.len(),
                    }))
                }
                WmDocAction::Get { path } => {
                    let doc_path = ensure_md_ext(&path);

                    let root = engine
                        .project_root
                        .read()
                        .map_err(|_| ToolError::lock_poisoned("project_root"))?
                        .clone();

                    let full_path = wiki_docs_dir(&root).join(&doc_path);

                    // Security: ensure path doesn't escape .wm/wiki/
                    if !full_path.starts_with(wiki_docs_dir(&root)) {
                        return Err(ToolError::internal("Path traversal detected"));
                    }

                    if !full_path.exists() || !full_path.is_file() {
                        return Err(ToolError::not_found("doc", &doc_path));
                    }

                    let content = std::fs::read_to_string(&full_path).map_err(|e| {
                        ToolError::io_error("read", full_path.to_string_lossy(), e)
                    })?;

                    let (frontmatter, body) = parse_frontmatter(&content);

                    let title = frontmatter
                        .get("title")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();

                    let tags: Vec<String> = frontmatter
                        .get("tags")
                        .and_then(|v| v.as_array())
                        .map(|arr| {
                            arr.iter()
                                .filter_map(|v| v.as_str().map(String::from))
                                .collect()
                        })
                        .unwrap_or_default();

                    Ok(serde_json::to_value(WmDocGetOutput {
                        path: path.clone(),
                        title,
                        content,
                        body,
                        frontmatter,
                        tags,
                    })
                    .unwrap_or(serde_json::Value::Null))
                }
                WmDocAction::Create {
                    path,
                    title,
                    content,
                    r#type: _,
                    tags,
                } => {
                    let doc_path = ensure_md_ext(&path);
                    let title = title;
                    let content = content.unwrap_or_default();
                    let tags = tags.unwrap_or_default();

                    let root = engine
                        .project_root
                        .read()
                        .map_err(|_| ToolError::lock_poisoned("project_root"))?
                        .clone();

                    let full_path = wiki_docs_dir(&root).join(&doc_path);

                    // Security: ensure path doesn't escape .wm/wiki/
                    if !full_path.starts_with(wiki_docs_dir(&root)) {
                        return Err(ToolError::internal("Path traversal detected"));
                    }

                    if full_path.exists() {
                        return Err(ToolError::internal(format!(
                            "Doc already exists: {}",
                            doc_path
                        )));
                    }

                    // Create parent directories
                    if let Some(parent) = full_path.parent() {
                        std::fs::create_dir_all(parent).map_err(|e| {
                            ToolError::io_error("create_dir", parent.to_string_lossy(), e)
                        })?;
                    }

                    let markdown = build_markdown(&title, &content, &tags);

                    std::fs::write(&full_path, &markdown).map_err(|e| {
                        ToolError::io_error("write", full_path.to_string_lossy(), e)
                    })?;

                    Ok(serde_json::to_value(WmDocCreateOutput {
                        path,
                        title,
                        tags,
                        status: "created".to_string(),
                    })
                    .unwrap_or(serde_json::Value::Null))
                }
                WmDocAction::Update {
                    path,
                    title,
                    content,
                } => {
                    let doc_path = ensure_md_ext(&path);
                    let new_title = title;
                    let new_content = content;

                    let root = engine
                        .project_root
                        .read()
                        .map_err(|_| ToolError::lock_poisoned("project_root"))?
                        .clone();

                    let full_path = wiki_docs_dir(&root).join(&doc_path);

                    // Security: ensure path doesn't escape .wm/wiki/
                    if !full_path.starts_with(wiki_docs_dir(&root)) {
                        return Err(ToolError::internal("Path traversal detected"));
                    }

                    if !full_path.exists() || !full_path.is_file() {
                        return Err(ToolError::not_found("doc", &doc_path));
                    }

                    let content =
                        std::fs::read_to_string(&full_path).map_err(|e| {
                            ToolError::io_error("read", full_path.to_string_lossy(), e)
                        })?;

                    let (mut frontmatter, body) = parse_frontmatter(&content);

                    if let Some(title) = new_title {
                        frontmatter.insert("title".to_string(), json!(title));
                    }

                    let final_body = new_content.unwrap_or(body);

                    let markdown = build_markdown_from_map(&frontmatter, &final_body);

                    std::fs::write(&full_path, &markdown).map_err(|e| {
                        ToolError::io_error("write", full_path.to_string_lossy(), e)
                    })?;

                    Ok(serde_json::to_value(WmDocUpdateOutput {
                        path,
                        status: "updated".to_string(),
                    })
                    .unwrap_or(serde_json::Value::Null))
                }
                WmDocAction::Delete { path } => {
                    let doc_path = ensure_md_ext(&path);

                    let root = engine
                        .project_root
                        .read()
                        .map_err(|_| ToolError::lock_poisoned("project_root"))?
                        .clone();

                    let full_path = wiki_docs_dir(&root).join(&doc_path);

                    // Security: ensure path doesn't escape .wm/wiki/
                    if !full_path.starts_with(wiki_docs_dir(&root)) {
                        return Err(ToolError::internal("Path traversal detected"));
                    }

                    if !full_path.exists() {
                        return Err(ToolError::not_found("doc", &doc_path));
                    }

                    std::fs::remove_file(&full_path).map_err(|e| {
                        ToolError::io_error("delete", full_path.to_string_lossy(), e)
                    })?;

                    Ok(serde_json::to_value(WmDocDeleteOutput {
                        path,
                        status: "deleted".to_string(),
                    })
                    .unwrap_or(serde_json::Value::Null))
                }
            }
        },
    );
}

/// Parse YAML frontmatter from markdown content.
/// Returns (frontmatter_map, body_text).
fn parse_frontmatter(
    content: &str,
) -> (serde_json::Map<String, serde_json::Value>, String) {
    let trimmed = content.trim();
    if !trimmed.starts_with("---") {
        return (serde_json::Map::new(), content.to_string());
    }

    let after_opening = &trimmed[3..];
    let end = match after_opening.find("\n---") {
        Some(pos) => pos,
        None => return (serde_json::Map::new(), content.to_string()),
    };

    let yaml_str = &trimmed[3..3 + end];
    let body = trimmed[3 + end + 4..].trim_start().to_string();

    let fm_value: serde_json::Value =
        serde_yaml::from_str(yaml_str).unwrap_or(serde_json::Value::Object(
            serde_json::Map::new(),
        ));
    let frontmatter = match fm_value {
        serde_json::Value::Object(map) => map,
        _ => serde_json::Map::new(),
    };

    (frontmatter, body)
}

/// Build markdown content with YAML frontmatter from title, body, and tags.
fn build_markdown(title: &str, content: &str, tags: &[String]) -> String {
    let mut fm = serde_json::Map::new();
    fm.insert("title".to_string(), json!(title));
    if !tags.is_empty() {
        fm.insert("tags".to_string(), json!(tags));
    }
    let yaml_str = serde_yaml::to_string(&fm).unwrap_or_default();
    format!("---\n{}---\n\n{}", yaml_str, content)
}

/// Build markdown content from an existing frontmatter map and body text.
fn build_markdown_from_map(
    frontmatter: &serde_json::Map<String, serde_json::Value>,
    body: &str,
) -> String {
    let yaml_str = serde_yaml::to_string(frontmatter).unwrap_or_default();
    format!("---\n{}---\n\n{}", yaml_str, body)
}
