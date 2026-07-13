use std::sync::Arc;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::engine::{EngineState, PageType};
use crate::error::ToolError;
use crate::mcp::transport::ToolRegistry;
use crate::mcp::typed::TypedRegister;
use crate::parser;

#[derive(Deserialize, JsonSchema)]
struct WmDecisionCreateInput {
    #[schemars(description = "Decision page ID")]
    id: String,
    #[schemars(description = "Decision title")]
    title: String,
    #[schemars(description = "Context (background/why)")]
    context: String,
    #[schemars(description = "Options considered")]
    options: Option<Vec<String>>,
    #[schemars(description = "Rationale for the chosen option")]
    rationale: String,
    #[schemars(description = "Outcome or result")]
    outcome: Option<String>,
    #[schemars(description = "Content body")]
    content: Option<String>,
    #[schemars(description = "Status: draft/accepted/superseded/rejected/archived")]
    status: Option<String>,
}

#[derive(Serialize)]
struct WmDecisionCreateOutput {
    id: String,
    title: String,
    status: String,
}

#[derive(Deserialize, JsonSchema)]
struct WmDecisionGetInput {
    #[schemars(description = "Decision page ID")]
    id: String,
}

pub fn register(registry: &mut ToolRegistry, engine: Arc<EngineState>) {
    let e = engine.clone();
    registry.register_write(
        "wm_decision.create",
        "Create a new architectural decision record (ADR). Stores context, options, rationale, and outcome.",
        move |input: WmDecisionCreateInput| {
            let id = input.id;
            let title = input.title;
            let status = input.status.unwrap_or_else(|| "draft".to_string());
            let content = input.content.unwrap_or_default();

            let mut frontmatter = format!(
                "title: {}\ntype: decision\nstatus: {}\n", title, status
            );
            frontmatter.push_str(&format!("decision:\n  context: \"{}\"\n", input.context));
            frontmatter.push_str(&format!("  rationale: \"{}\"\n", input.rationale));
            if let Some(opts) = input.options {
                if !opts.is_empty() {
                    frontmatter.push_str(&format!("  options: [{}]\n", opts.iter().map(|o| format!("\"{}\"", o)).collect::<Vec<_>>().join(", ")));
                }
            }
            if let Some(outcome) = input.outcome {
                frontmatter.push_str(&format!("  outcome: \"{}\"\n", outcome));
            }

            let _ = crate::page::create_page(&e, &id, &frontmatter, &content)?;
            Ok(WmDecisionCreateOutput { id, title, status })
        },
    );

    let e = engine.clone();
    registry.register_read(
        "wm_decision.get",
        "Get a decision record by ID.",
        move |input: WmDecisionGetInput| {
            let snapshot = e.graph.load();
            let index = &snapshot.1;
            let node_idx = index.get(&input.id)
                .ok_or_else(|| ToolError::not_found("decision", &input.id))?;
            let meta = &snapshot.0[*node_idx];

            if meta.page_type != PageType::Decision {
                return Err(ToolError::not_found("decision", &input.id));
            }

            let content = std::fs::read_to_string(&meta.path)
                .map_err(|e| ToolError::io_error("read", meta.path.to_string_lossy(), e))?;
            let (_fm, body) = parser::extract_frontmatter(&content);

            Ok(json!({
                "id": meta.id,
                "title": meta.title,
                "status": meta.status.as_str(),
                "context": meta.decision.as_ref().map(|d| &d.context),
                "options": meta.decision.as_ref().map(|d| &d.options),
                "rationale": meta.decision.as_ref().map(|d| &d.rationale),
                "outcome": meta.decision.as_ref().map(|d| &d.outcome),
                "content": body,
            }))
        },
    );
}
