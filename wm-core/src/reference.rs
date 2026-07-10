// ─── Reference System — Inline @doc/, @task/, @memory/ Resolution ─

use regex::Regex;
use serde::Serialize;

use crate::engine::EngineState;
use crate::error::ToolError;

/// Compiled regex for extracting @references, cached once via LazyLock.
static REFERENCE_RE: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
    Regex::new(r"\B@(doc|task|memory|decision|template)/([A-Za-z0-9_./-]+)")
        .expect("valid reference regex")
});

/// A parsed reference from body text.
#[derive(Debug, Clone, Serialize)]
pub struct Reference {
    /// Reference type: doc, task, memory, decision, template
    pub ref_type: String,
    /// Path or ID within that type
    pub target: String,
    /// The full matched text (e.g., "@doc/specs/foo")
    pub full_match: String,
}

/// Extract inline @references from markdown body text.
/// Supports: @doc/path, @task/id, @memory/id, @decision/path, @template/name
/// Skips references inside code blocks (```...```).
pub fn extract_references(content: &str) -> Vec<Reference> {
    let mut refs = Vec::new();
    let mut in_code_block = false;

    for line in content.lines() {
        if line.trim_start().starts_with("```") {
            in_code_block = !in_code_block;
            continue;
        }
        if in_code_block {
            continue;
        }

        for cap in REFERENCE_RE.captures_iter(line) {
            let ref_type = cap[1].to_string();
            let target = cap[2].to_string();
            let full_match = format!("@{}/{}", ref_type, target);
            refs.push(Reference {
                ref_type,
                target,
                full_match,
            });
        }
    }

    refs
}

/// Resolve a single reference to its target content.
/// Returns markdown-formatted content of the referenced entity.
pub fn resolve_reference(
    reference: &Reference,
    engine: &EngineState,
) -> Result<String, ToolError> {
    match reference.ref_type.as_str() {
        "doc" | "decision" => {
            // Resolve wiki page path
            let page_path = reference.target.trim_end_matches(".md");
            let snapshot = engine.graph.load();
            let index = &snapshot.1;

            // Try exact ID match first
            if let Some(node_idx) = index.get(page_path) {
                let meta = &snapshot.0[*node_idx];
                let content = std::fs::read_to_string(&meta.path)
                    .map_err(|e| ToolError::io_error("read", meta.path.to_string_lossy(), e))?;
                return Ok(format!("# {}\n\n{}", meta.title, content));
            }

            // Try path-with-extension
            let md_path = if page_path.ends_with(".md") {
                page_path.to_string()
            } else {
                format!("{}.md", page_path)
            };
            if let Some(node_idx) = index.get(&md_path) {
                let meta = &snapshot.0[*node_idx];
                let content = std::fs::read_to_string(&meta.path)
                    .map_err(|e| ToolError::io_error("read", meta.path.to_string_lossy(), e))?;
                return Ok(format!("# {}\n\n{}", meta.title, content));
            }

            Err(ToolError::not_found("reference", &reference.full_match))
        }
        "task" => {
            resolve_wiki_page(&reference.target, engine)
        }
        "memory" => {
            // Read memory from project or session layer
            let root = engine.project_root.read()
                .map_err(|_| ToolError::lock_poisoned("project_root"))?
                .clone();
            let base_dir = root.join(".wm").join("memory");
            let target = sanitize_ref_target(&reference.target);
            let memory_file = base_dir.join(format!("{}.json", target));

            // Security: check path stays within expected directory
            if let Ok(canon) = memory_file.canonicalize() {
                if !canon.starts_with(&base_dir) {
                    return Err(ToolError::internal("Path traversal detected in memory reference"));
                }
            }

            if memory_file.exists() {
                let content = std::fs::read_to_string(&memory_file)
                    .map_err(|e| ToolError::io_error("read", memory_file.to_string_lossy(), e))?;
                return Ok(format!("```json\n{}\n```", content));
            }

            // Try session memory
            if let Some(entry) = engine.session_memory.get(&target) {
                return Ok(format!("**{}**\n\n{}", entry.title, entry.content));
            }

            Err(ToolError::not_found("reference", &reference.full_match))
        }
        "template" => {
            // Templates are stored in .wm/templates/<name>.json
            let root = engine.project_root.read()
                .map_err(|_| ToolError::lock_poisoned("project_root"))?
                .clone();
            let base_dir = root.join(".wm").join("templates");
            let target = sanitize_ref_target(&reference.target);
            let template_file = base_dir.join(format!("{}.json", target));

            // Security: check path stays within expected directory
            if let Ok(canon) = template_file.canonicalize() {
                if !canon.starts_with(&base_dir) {
                    return Err(ToolError::internal("Path traversal detected in template reference"));
                }
            }

            if template_file.exists() {
                let content = std::fs::read_to_string(&template_file)
                    .map_err(|e| ToolError::io_error("read", template_file.to_string_lossy(), e))?;
                return Ok(format!("```json\n{}\n```", content));
            }

            Err(ToolError::not_found("reference", &reference.full_match))
        }
        other => Err(ToolError::internal(format!(
            "Unknown reference type: @{}. Supported: doc, task, memory, decision, template",
            other
        ))),
    }
}

/// Resolve a wiki page by ID (used for both doc/* and task/* refs).
fn resolve_wiki_page(page_id: &str, engine: &EngineState) -> Result<String, ToolError> {
    let page_path = page_id.trim_end_matches(".md");
    let snapshot = engine.graph.load();
    let index = &snapshot.1;

    if let Some(node_idx) = index.get(page_path) {
        let meta = &snapshot.0[*node_idx];
        let content = std::fs::read_to_string(&meta.path)
            .map_err(|e| ToolError::io_error("read", meta.path.to_string_lossy(), e))?;
        return Ok(format!("# {}\n\n{}", meta.title, content));
    }

    Err(ToolError::not_found("reference", page_id))
}

/// Sanitize a reference target to prevent path traversal.
/// Strips ".." components and leading slashes.
fn sanitize_ref_target(target: &str) -> String {
    target
        .split('/')
        .filter(|&segment| segment != ".." && segment != ".")
        .collect::<Vec<_>>()
        .join("/")
}

/// Extract all references from content and resolve them.
/// Returns a map of full_match → resolved content.
pub fn resolve_all(
    content: &str,
    engine: &EngineState,
) -> Vec<(Reference, Result<String, ToolError>)> {
    let refs = extract_references(content);
    refs.into_iter()
        .map(|r| {
            let result = resolve_reference(&r, engine);
            (r, result)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_references_basic() {
        let content = "See @doc/specs/auth for details and @task/fix-bug for progress.";
        let refs = extract_references(content);
        assert_eq!(refs.len(), 2);
        assert_eq!(refs[0].ref_type, "doc");
        assert_eq!(refs[0].target, "specs/auth");
        assert_eq!(refs[1].ref_type, "task");
        assert_eq!(refs[1].target, "fix-bug");
    }

    #[test]
    fn test_extract_references_skips_code_blocks() {
        let content = "Normal text @doc/valid\n\n```\n@doc/should-skip\n```\n\nAfter @task/also-valid";
        let refs = extract_references(content);
        assert_eq!(refs.len(), 2);
        assert_eq!(refs[0].target, "valid");
        assert_eq!(refs[1].target, "also-valid");
    }

    #[test]
    fn test_extract_references_all_types() {
        let content = "@doc/a @task/b @memory/c @decision/d @template/e";
        let refs = extract_references(content);
        assert_eq!(refs.len(), 5);
        assert_eq!(refs[0].ref_type, "doc");
        assert_eq!(refs[1].ref_type, "task");
        assert_eq!(refs[2].ref_type, "memory");
        assert_eq!(refs[3].ref_type, "decision");
        assert_eq!(refs[4].ref_type, "template");
    }

    #[test]
    fn test_extract_references_no_refs() {
        let content = "Just plain text with no references.";
        let refs = extract_references(content);
        assert!(refs.is_empty());
    }

    #[test]
    fn test_extract_references_empty() {
        let refs = extract_references("");
        assert!(refs.is_empty());
    }

    #[test]
    fn test_reference_struct_fields() {
        let refs = extract_references("@doc/learnings/foo-bar");
        assert_eq!(refs.len(), 1);
        assert_eq!(refs[0].full_match, "@doc/learnings/foo-bar");
        assert_eq!(refs[0].ref_type, "doc");
        assert_eq!(refs[0].target, "learnings/foo-bar");
    }
}
