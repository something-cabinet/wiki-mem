use std::sync::Arc;

use crate::engine::EngineState;
use crate::error::ToolError;
use crate::mcp::handler::ToolArgs;
use crate::mcp::transport::ToolRegistry;

/// Register doc tool handlers
pub fn register(registry: &mut ToolRegistry, engine: Arc<EngineState>) {
    let e = engine.clone();
    registry.register_with_desc(
        "wm_doc.list",
        "List documents in the Knowns wiki (.knowns/docs/)",
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

                // Extract title from frontmatter or filename
                let title = extract_title(&path).unwrap_or_else(|| {
                    path.file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("")
                        .to_string()
                });

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
                    "filename": path.file_name().and_then(|s| s.to_str()).unwrap_or(""),
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
}

/// Extract title from markdown frontmatter
fn extract_title(path: &std::path::Path) -> Option<String> {
    let content = std::fs::read_to_string(path).ok()?;
    let content = content.trim();

    if !content.starts_with("---") {
        return None;
    }

    // Find end of frontmatter
    let end = content[3..].find("\n---")?;
    let frontmatter = &content[3..3 + end];

    for line in frontmatter.lines() {
        if let Some(value) = line.strip_prefix("title:") {
            let title = value.trim().trim_matches('"').trim_matches('\'').to_string();
            if !title.is_empty() {
                return Some(title);
            }
        }
    }

    None
}
