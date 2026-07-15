use std::sync::Arc;

use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::json;

use crate::engine::EngineState;
use crate::error::ToolError;
use crate::mcp::transport::ToolRegistry;


#[derive(Deserialize, JsonSchema)]
struct WmRefExtractInput {
    #[schemars(description = "Markdown content to extract references from")]
    content: String,
}

#[derive(Deserialize, JsonSchema)]
struct WmRefResolveInput {
    #[schemars(description = "Full reference string (e.g., @wiki/tasks/fix-bug)")]
    reference: String,
}

#[derive(Deserialize, JsonSchema)]
struct WmRefResolveAllInput {
    #[schemars(description = "Markdown content containing @references to resolve")]
    content: String,
}

/// Register reference tool handlers
pub fn register(registry: &mut ToolRegistry, engine: Arc<EngineState>) {
    // wm_ref.extract — extract all @references from content (no engine needed)
    registry.register_typed(
        "wm_ref.extract",
        "Extract all @wiki/{type}/{name} references from markdown content. Skips code blocks.",
        move |input: WmRefExtractInput| {
            let refs = crate::reference::extract_references(&input.content);
            Ok(json!({
                "references": refs,
                "count": refs.len(),
            }))
        },
    );

    // wm_ref.resolve — resolve a single @reference to its content
    let e1 = engine.clone();
    registry.register_typed(
        "wm_ref.resolve",
        "Resolve a single @reference string (e.g., @wiki/tasks/fix-bug) to its target content.",
        move |input: WmRefResolveInput| {
            let refs = crate::reference::extract_references(&input.reference);
            if refs.is_empty() {
                return Err(ToolError::internal(format!(
                    "No valid reference found in '{}'. Expected format: @wiki/{{type}}/{{name}}",
                    input.reference
                )));
            }
            let reference = &refs[0];
            let content = crate::reference::resolve_reference(reference, &e1)?;
            Ok(json!({
                "reference": &reference.full_match,
                "ref_type": &reference.ref_type,
                "target": &reference.target,
                "content": content,
            }))
        },
    );

    // wm_ref.resolve_all — resolve all @references in a body of text
    let e2 = engine;
    registry.register_typed(
        "wm_ref.resolve_all",
        "Extract and resolve all @references in markdown content. Returns a list of resolved references.",
        move |input: WmRefResolveAllInput| {
            let results = crate::reference::resolve_all_references(&input.content, &e2);
            let mut resolved = Vec::new();
            let mut errors = Vec::new();

            for (reference, result) in results {
                match result {
                    Ok(content) => resolved.push(json!({
                        "reference": &reference.full_match,
                        "ref_type": &reference.ref_type,
                        "target": &reference.target,
                        "content": content,
                        "resolved": true,
                    })),
                    Err(err) => errors.push(json!({
                        "reference": &reference.full_match,
                        "ref_type": &reference.ref_type,
                        "target": &reference.target,
                        "error": err.to_string(),
                        "resolved": false,
                    })),
                }
            }

            Ok(json!({
                "resolved": resolved,
                "errors": errors,
                "total": resolved.len() + errors.len(),
                "resolved_count": resolved.len(),
                "error_count": errors.len(),
            }))
        },
    );
}
