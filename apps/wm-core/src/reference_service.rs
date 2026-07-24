
use serde::Serialize;

use crate::reference_constant::REFERENCE_RE;

use crate::engine::EngineState;
use crate::error::ToolError;

#[derive(Debug, Clone, Serialize)]
pub struct Reference {
    pub ref_type: String,
    pub target: String,
    pub full_match: String,
}

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
            let full_match = format!("@wiki/{}/{}", ref_type, target);
            refs.push(Reference {
                ref_type,
                target,
                full_match,
            });
        }
    }

    refs
}

pub fn resolve_reference(
    reference: &Reference,
    engine: &EngineState,
) -> Result<String, ToolError> {
    match reference.ref_type.as_str() {
        "tasks" | "specs" | "concepts" | "patterns" | "decisions" | "rules" | "memory" | "howto" | "reference" | "notes" | "core" => {
            let page_id = format!("wiki:{}:{}", reference.ref_type, reference.target);
            resolve_wiki_page(&page_id, engine)
        }
        "templates" => {
            let root = engine.project_root.read()
                .map_err(|_| ToolError::lock_poisoned("project_root"))?
                .clone();
            let base_dir = root.join(".wm").join("templates");
            let target = sanitize_ref_target(&reference.target);
            let template_file = base_dir.join(format!("{}.json", target));

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
            "Unknown reference type in @wiki reference: {}. Supported wiki directories: tasks, specs, concepts, patterns, decisions, memory, howto, reference, notes. Also supports: templates.",
            other
        ))),
    }
}

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

fn sanitize_ref_target(target: &str) -> String {
    target
        .split('/')
        .filter(|&segment| segment != ".." && segment != ".")
        .fold(
            String::new(),
            |mut acc, segment| {
                if !acc.is_empty() { acc.push('/'); }
                acc.push_str(segment);
                acc
            },
        )
}

pub fn resolve_all_references(
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
        let content = "See @wiki/specs/auth for details and @wiki/tasks/fix-bug for progress.";
        let refs = extract_references(content);
        assert_eq!(refs.len(), 2);
        assert_eq!(refs[0].ref_type, "specs");
        assert_eq!(refs[0].target, "auth");
        assert_eq!(refs[1].ref_type, "tasks");
        assert_eq!(refs[1].target, "fix-bug");
    }

    #[test]
    fn test_extract_references_skips_code_blocks() {
        let content = "Normal text @wiki/specs/valid\n\n```\n@wiki/specs/should-skip\n```\n\nAfter @wiki/tasks/also-valid";
        let refs = extract_references(content);
        assert_eq!(refs.len(), 2);
        assert_eq!(refs[0].target, "valid");
        assert_eq!(refs[1].target, "also-valid");
    }

    #[test]
    fn test_extract_references_all_types() {
        let content = "@wiki/specs/a @wiki/tasks/b @wiki/memory/c @wiki/decisions/d @wiki/templates/e";
        let refs = extract_references(content);
        assert_eq!(refs.len(), 5);
        assert_eq!(refs[0].ref_type, "specs");
        assert_eq!(refs[1].ref_type, "tasks");
        assert_eq!(refs[2].ref_type, "memory");
        assert_eq!(refs[3].ref_type, "decisions");
        assert_eq!(refs[4].ref_type, "templates");
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
        let refs = extract_references("@wiki/specs/learnings/foo-bar");
        assert_eq!(refs.len(), 1);
        assert_eq!(refs[0].full_match, "@wiki/specs/learnings/foo-bar");
        assert_eq!(refs[0].ref_type, "specs");
        assert_eq!(refs[0].target, "learnings/foo-bar");
    }
}
