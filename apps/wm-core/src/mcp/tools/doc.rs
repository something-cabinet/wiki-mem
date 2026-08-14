use crate::engine::PageType;
use crate::mcp::prelude::*;
use crate::page::helpers::yaml_helper::{remove_yaml_block, set_yaml_field, yaml_scalar};
use serde::Serialize;
use wm_constants::*;

use std::path::PathBuf;

fn wiki_docs_dir(root: &std::path::Path) -> PathBuf {
    root.join(WM_DIR).join(WIKI_DIR)
}

fn ensure_md_ext(path: &str) -> String {
    if path.ends_with(".md") {
        path.to_string()
    } else {
        format!("{}.md", path)
    }
}


#[derive(Deserialize, JsonSchema)]
#[serde(tag = "action", rename_all = "snake_case")]
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
        r#type: Option<String>,
        tags: Option<Vec<String>>,
    },
    #[schemars(description = "Delete a doc")]
    Delete { path: String },
}


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

pub fn register(registry: &mut ToolRegistry, engine: Arc<EngineState>) {
    registry.register_typed_async(
        "wm_doc",
        "Doc CRUD operations: list, get, create, update, delete",
        move |input: WmDocAction| {
            let engine = engine.clone();
            async move {
                match input {
                    WmDocAction::List { r#type } => {
                        let folder = r#type;
                        let root = engine
                            .project_root
                            .read()
                            .map_err(|_| ToolError::lock_poisoned("project_root"))?
                            .clone();
                        tokio::task::spawn_blocking(move || list_docs(&root, folder.as_deref()))
                            .await
                            .map_err(|e| ToolError::internal(format!("doc list task failed: {e}")))?
                    }
                    WmDocAction::Get { path } => {
                        let doc_path = ensure_md_ext(&path);

                        let root = engine
                            .project_root
                            .read()
                            .map_err(|_| ToolError::lock_poisoned("project_root"))?
                            .clone();

                        let full_path = crate::shared::helpers::path_confine_helper::confine(
                            &wiki_docs_dir(&root),
                            std::path::Path::new(&doc_path),
                        )?;

                        let meta = tokio::fs::metadata(&full_path)
                            .await
                            .map_err(|_| ToolError::not_found("doc", &doc_path))?;
                        if !meta.is_file() {
                            return Err(ToolError::not_found("doc", &doc_path));
                        }

                        let content =
                            tokio::fs::read_to_string(&full_path).await.map_err(|e| {
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
                        r#type,
                        tags,
                    } => {
                        let doc_path = ensure_md_ext(&path);
                        let content = content.unwrap_or_default();
                        let tags = tags.unwrap_or_default();
                        let root = engine
                            .project_root
                            .read()
                            .map_err(|_| ToolError::lock_poisoned("project_root"))?
                            .clone();

                        let full_path = crate::shared::helpers::path_confine_helper::confine(
                            &wiki_docs_dir(&root),
                            std::path::Path::new(&doc_path),
                        )?;

                        if tokio::fs::metadata(&full_path).await.is_ok() {
                            return Err(ToolError::internal(format!(
                                "Doc already exists: {}",
                                doc_path
                            )));
                        }

                        if let Some(parent) = full_path.parent() {
                            tokio::fs::create_dir_all(parent).await.map_err(|e| {
                                ToolError::io_error("create_dir", parent.to_string_lossy(), e)
                            })?;
                        }

                        // FR-3: mirror wm_page's default — when `type` is absent,
                        // derive it from the path's first directory segment.
                        let resolved_type = r#type.unwrap_or_else(|| {
                            let first_segment = path
                                .trim_start_matches("wiki/")
                                .split('/')
                                .next()
                                .unwrap_or("concept");
                            PageType::from_dir_name(first_segment)
                                .unwrap_or(PageType::Concept)
                                .as_str()
                                .to_string()
                        });

                        let markdown = build_markdown(&title, &resolved_type, &content, &tags);

                        tokio::fs::write(&full_path, &markdown).await.map_err(|e| {
                            ToolError::io_error("write", full_path.to_string_lossy(), e)
                        })?;

                        Ok(serde_json::to_value(WmDocCreateOutput {
                            path,
                            title,
                            tags,
                            status: "created".into(),
                        })
                        .unwrap_or(serde_json::Value::Null))
                    }
                    WmDocAction::Update {
                        path,
                        title,
                        content,
                        r#type,
                        tags,
                    } => {
                        let doc_path = ensure_md_ext(&path);

                        let root = engine
                            .project_root
                            .read()
                            .map_err(|_| ToolError::lock_poisoned("project_root"))?
                            .clone();

                        let full_path = crate::shared::helpers::path_confine_helper::confine(
                            &wiki_docs_dir(&root),
                            std::path::Path::new(&doc_path),
                        )?;

                        let meta = tokio::fs::metadata(&full_path)
                            .await
                            .map_err(|_| ToolError::not_found("doc", &doc_path))?;
                        if !meta.is_file() {
                            return Err(ToolError::not_found("doc", &doc_path));
                        }

                        let file_content =
                            tokio::fs::read_to_string(&full_path).await.map_err(|e| {
                                ToolError::io_error("read", full_path.to_string_lossy(), e)
                            })?;

                        // Merge line-wise into the raw frontmatter block (same
                        // helpers as wm_page.update) so title/type/tags are
                        // byte-identical to wm_page output and every other
                        // frontmatter field is preserved untouched.
                        let (existing_fm, body) = crate::parser::extract_frontmatter(&file_content);
                        let (raw_fm_opt, _) = crate::parser::extract_raw_frontmatter(&file_content);
                        let mut new_fm = match raw_fm_opt {
                            Some(raw) => raw,
                            None => existing_fm
                                .as_ref()
                                .map(crate::parser::frontmatter_to_yaml)
                                .unwrap_or_default(),
                        };

                        if let Some(title) = title {
                            new_fm = set_yaml_field(&new_fm, "title", &title);
                        }

                        if let Some(r#type) = r#type {
                            new_fm = set_yaml_field(&new_fm, "type", &r#type);
                        }

                        if let Some(ref tag_list) = tags {
                            new_fm = remove_yaml_block(&new_fm, "tags");
                            if !tag_list.is_empty() {
                                new_fm.push_str(&format!("tags: [{}]\n", tag_list.join(", ")));
                            }
                        }

                        let final_body = content.unwrap_or_else(|| body.to_string());

                        let markdown = format!("---\n{}---\n\n{}", new_fm, final_body);

                        tokio::fs::write(&full_path, &markdown).await.map_err(|e| {
                            ToolError::io_error("write", full_path.to_string_lossy(), e)
                        })?;

                        Ok(serde_json::to_value(WmDocUpdateOutput {
                            path,
                            status: "updated".into(),
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

                        let full_path = crate::shared::helpers::path_confine_helper::confine(
                            &wiki_docs_dir(&root),
                            std::path::Path::new(&doc_path),
                        )?;

                        if tokio::fs::metadata(&full_path).await.is_err() {
                            return Err(ToolError::not_found("doc", &doc_path));
                        }

                        tokio::fs::remove_file(&full_path).await.map_err(|e| {
                            ToolError::io_error("delete", full_path.to_string_lossy(), e)
                        })?;

                        Ok(serde_json::to_value(WmDocDeleteOutput {
                            path,
                            status: "deleted".into(),
                        })
                        .unwrap_or(serde_json::Value::Null))
                    }
                }
            }
        },
    );
}

/// Blocking doc listing, run on a blocking thread pool from the async handler.
fn list_docs(
    root: &std::path::Path,
    folder: Option<&str>,
) -> Result<serde_json::Value, ToolError> {
    let wiki_dir = wiki_docs_dir(root);
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

        let content = std::fs::read_to_string(&path).unwrap_or_default();
        let (frontmatter, _body) = parse_frontmatter(&content);

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

        let tags: Vec<String> = frontmatter
            .get("tags")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();

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

        let doc_folder = path
            .parent()
            .and_then(|p| p.strip_prefix(&wiki_dir).ok())
            .map(|p| p.to_string_lossy().to_string())
            .filter(|s| !s.is_empty())
            .unwrap_or_default();

        let doc_path = path
            .strip_prefix(root)
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

/// Build a markdown doc whose frontmatter is byte-identical to what
/// `wm_page.create` writes for the shared title/type/tags fields (AC-4 parity).
/// `type` mirrors `wm_page`'s `type: <scalar>` line and `tags` its inline
/// `tags: [a, b]` flow form — never serde_yaml block-form lists.
fn build_markdown(title: &str, r#type: &str, content: &str, tags: &[String]) -> String {
    let mut fm = String::new();
    fm.push_str(&format!("title: {}\n", yaml_scalar(title)));
    fm.push_str(&format!("type: {}\n", yaml_scalar(r#type)));
    if !tags.is_empty() {
        fm.push_str(&format!("tags: [{}]\n", tags.join(", ")));
    }
    format!("---\n{}---\n\n{}", fm, content)
}
