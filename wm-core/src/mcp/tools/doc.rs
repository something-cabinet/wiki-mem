use std::sync::Arc;

use serde_json::json;

use crate::engine::EngineState;
use crate::error::ToolError;
use crate::mcp::handler::ToolArgs;
use crate::mcp::transport::ToolRegistry;

/// Register doc tool handlers
pub fn register(registry: &mut ToolRegistry, engine: Arc<EngineState>) {
    // ─── wm_doc.list ───────────────────────────────────────────
    let e = engine.clone();
    registry.register_with_schema(
        "doc.list",
        "List documents in the Knowns wiki (.knowns/docs/)",
        json!({
            "type": "object",
            "properties": {
                "folder": { "type": "string", "description": "Subfolder path to list" }
            }
        }),
        Arc::new(move |params| {
            let args = ToolArgs::new(params);
            let folder = args.optional_string("folder");
            let _pattern = args.optional_string("pattern");

            let root = e
                .project_root
                .read()
                .map_err(|_| ToolError::lock_poisoned("project_root"))?
                .clone();

            let knowns_docs = root.join(".knowns").join("docs");
            if !knowns_docs.exists() || !knowns_docs.is_dir() {
                return Ok(serde_json::json!({
                    "docs": [],
                    "total": 0,
                    "path": knowns_docs.to_string_lossy(),
                    "note": ".knowns/docs/ not found"
                }));
            }

            let walk_dir = match &folder {
                Some(f) => knowns_docs.join(f),
                None => knowns_docs.clone(),
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
                let content = std::fs::read_to_string(&path).unwrap_or_default();
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

                // Determine folder relative to .knowns/docs/
                let doc_folder = path
                    .parent()
                    .and_then(|p| p.strip_prefix(&knowns_docs).ok())
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
        }),
    );

    // ─── wm_doc.get ────────────────────────────────────────────
    let e = engine.clone();
    registry.register_with_schema(
        "doc.get",
        "Read a doc from .knowns/docs/ by path",
        json!({
            "type": "object",
            "properties": {
                "path": { "type": "string", "description": "Doc path" }
            },
            "required": ["path"]
        }),
        Arc::new(move |params| {
            let args = ToolArgs::new(params);
            let doc_path = args.require_string("path")?;

            let root = e
                .project_root
                .read()
                .map_err(|_| ToolError::lock_poisoned("project_root"))?
                .clone();

            let full_path = root.join(".knowns").join("docs").join(&doc_path);

            // Security: ensure path doesn't escape .knowns/docs/
            if !full_path.starts_with(root.join(".knowns").join("docs")) {
                return Err(ToolError::internal("Path traversal detected"));
            }

            if !full_path.exists() || !full_path.is_file() {
                return Err(ToolError::not_found("doc", &doc_path));
            }

            let content = std::fs::read_to_string(&full_path)
                .map_err(|e| ToolError::io_error("read", full_path.to_string_lossy(), e))?;

            let (frontmatter, body) = parse_frontmatter(&content);

            let title = frontmatter
                .get("title")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            let tags: Vec<String> = frontmatter
                .get("tags")
                .and_then(|v| v.as_array())
                .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                .unwrap_or_default();

            Ok(json!({
                "path": doc_path,
                "title": title,
                "content": content,
                "body": body,
                "frontmatter": frontmatter,
                "tags": tags,
            }))
        }),
    );

    // ─── wm_doc.create ─────────────────────────────────────────
    let e = engine.clone();
    registry.register_with_schema(
        "doc.create",
        "Create a new doc in .knowns/docs/",
        json!({
            "type": "object",
            "properties": {
                "path": { "type": "string", "description": "Doc path" },
                "title": { "type": "string", "description": "Doc title" },
                "content": { "type": "string", "description": "Doc content" },
                "tags": { "type": "array", "items": { "type": "string" }, "description": "Tags" }
            },
            "required": ["path", "title"]
        }),
        Arc::new(move |params| {
            let args = ToolArgs::new(params);
            let doc_path = args.require_string("path")?;
            let title = args.require_string("title")?;
            let content = args.optional_string("content").unwrap_or_default();
            let tags = args.optional_string_array("tags");

            let root = e
                .project_root
                .read()
                .map_err(|_| ToolError::lock_poisoned("project_root"))?
                .clone();

            let full_path = root.join(".knowns").join("docs").join(&doc_path);

            // Security: ensure path doesn't escape .knowns/docs/
            if !full_path.starts_with(root.join(".knowns").join("docs")) {
                return Err(ToolError::internal("Path traversal detected"));
            }

            if full_path.exists() {
                return Err(ToolError::internal(format!("Doc already exists: {}", doc_path)));
            }

            // Create parent directories
            if let Some(parent) = full_path.parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| ToolError::io_error("create_dir", parent.to_string_lossy(), e))?;
            }

            let markdown = build_markdown(&title, &content, &tags);

            std::fs::write(&full_path, &markdown)
                .map_err(|e| ToolError::io_error("write", full_path.to_string_lossy(), e))?;

            Ok(json!({
                "path": doc_path,
                "title": title,
                "tags": tags,
                "status": "created"
            }))
        }),
    );

    // ─── wm_doc.update ─────────────────────────────────────────
    let e = engine.clone();
    registry.register_with_schema(
        "doc.update",
        "Update an existing doc",
        json!({
            "type": "object",
            "properties": {
                "path": { "type": "string", "description": "Doc path" },
                "title": { "type": "string", "description": "New title" },
                "content": { "type": "string", "description": "New content" },
                "tags": { "type": "array", "items": { "type": "string" }, "description": "New tags" }
            },
            "required": ["path"]
        }),
        Arc::new(move |params| {
            let params_clone = params.clone();
            let args = ToolArgs::new(params);
            let doc_path = args.require_string("path")?;
            let new_title = args.optional_string("title");
            let new_content = args.optional_string("content");
            let new_tags = args.optional_string_array("tags");
            let has_new_tags = params_clone.get("tags").is_some();

            let root = e
                .project_root
                .read()
                .map_err(|_| ToolError::lock_poisoned("project_root"))?
                .clone();

            let full_path = root.join(".knowns").join("docs").join(&doc_path);

            // Security: ensure path doesn't escape .knowns/docs/
            if !full_path.starts_with(root.join(".knowns").join("docs")) {
                return Err(ToolError::internal("Path traversal detected"));
            }

            if !full_path.exists() || !full_path.is_file() {
                return Err(ToolError::not_found("doc", &doc_path));
            }

            let content = std::fs::read_to_string(&full_path)
                .map_err(|e| ToolError::io_error("read", full_path.to_string_lossy(), e))?;

            let (mut frontmatter, body) = parse_frontmatter(&content);

            if let Some(title) = new_title {
                frontmatter.insert("title".to_string(), json!(title));
            }

            if has_new_tags {
                frontmatter.insert("tags".to_string(), json!(new_tags));
            }

            let final_body = new_content.unwrap_or(body);

            let markdown = build_markdown_from_map(&frontmatter, &final_body);

            std::fs::write(&full_path, &markdown)
                .map_err(|e| ToolError::io_error("write", full_path.to_string_lossy(), e))?;

            Ok(json!({
                "path": doc_path,
                "status": "updated"
            }))
        }),
    );

    // ─── wm_doc.delete ─────────────────────────────────────────
    let e = engine.clone();
    registry.register_with_schema(
        "doc.delete",
        "Delete a doc",
        json!({
            "type": "object",
            "properties": {
                "path": { "type": "string", "description": "Doc path" }
            },
            "required": ["path"]
        }),
        Arc::new(move |params| {
            let args = ToolArgs::new(params);
            let doc_path = args.require_string("path")?;

            let root = e
                .project_root
                .read()
                .map_err(|_| ToolError::lock_poisoned("project_root"))?
                .clone();

            let full_path = root.join(".knowns").join("docs").join(&doc_path);

            // Security: ensure path doesn't escape .knowns/docs/
            if !full_path.starts_with(root.join(".knowns").join("docs")) {
                return Err(ToolError::internal("Path traversal detected"));
            }

            if !full_path.exists() {
                return Err(ToolError::not_found("doc", &doc_path));
            }

            std::fs::remove_file(&full_path)
                .map_err(|e| ToolError::io_error("delete", full_path.to_string_lossy(), e))?;

            Ok(json!({
                "path": doc_path,
                "status": "deleted"
            }))
        }),
    );
}

/// Parse YAML frontmatter from markdown content.
/// Returns (frontmatter_map, body_text).
fn parse_frontmatter(content: &str) -> (serde_json::Map<String, serde_json::Value>, String) {
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
        serde_yaml::from_str(yaml_str).unwrap_or(serde_json::Value::Object(serde_json::Map::new()));
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
