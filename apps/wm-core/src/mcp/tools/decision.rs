use crate::mcp::prelude::*;
use serde_json::json;
use crate::engine::PageType;

use crate::parser;
use crate::status::PageStatus;

// ─── Action enum ─────────────────────────────────────────────

#[derive(Deserialize, JsonSchema)]
#[serde(tag = "action", rename_all = "snake_case")]
enum WmDecisionAction {
    Create {
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
        status: Option<PageStatus>,
    },
    Get {
        #[schemars(description = "Decision page ID")]
        id: String,
    },
}

pub fn register(registry: &mut ToolRegistry, engine: Arc<EngineState>) {
    let e = engine.clone();
    registry.register_typed(
        "wm_decision",
        "Manage architectural decision records (create, get)",
        move |input: WmDecisionAction| {
            match input {
                WmDecisionAction::Create { id, title, context, options, rationale, outcome, content, status } => {
                    let page_status = status.unwrap_or(PageStatus::Draft);
                    if !PageType::Decision.allowed_statuses().contains(&page_status) {
                        return Err(ToolError::invalid_params(format!(
                            "Invalid status '{}' for decision page. Allowed: {}",
                            page_status,
                            PageType::Decision.allowed_statuses().iter().map(|s| s.as_str()).collect::<Vec<_>>().join(", ")
                        )));
                    }
                    let status = page_status.as_str().to_string();
                    let content = content.unwrap_or_default();

                    let mut frontmatter = format!(
                        "title: {}\ntype: decision\nstatus: {}\n", title, status
                    );
                    frontmatter.push_str(&format!("decision:\n  context: \"{}\"\n", context));
                    frontmatter.push_str(&format!("  rationale: \"{}\"\n", rationale));
                    if let Some(opts) = options {
                        if !opts.is_empty() {
                            frontmatter.push_str(&format!("  options: [{}]\n", opts.iter().map(|o| format!("\"{}\"", o)).collect::<Vec<_>>().join(", ")));
                        }
                    }
                    if let Some(outcome) = outcome {
                        frontmatter.push_str(&format!("  outcome: \"{}\"\n", outcome));
                    }

                    let _ = crate::page::create_page(&e, &id, &frontmatter, &content)?;
                    Ok(json!({
                        "id": id,
                        "title": title,
                        "status": status,
                    }))
                }

                WmDecisionAction::Get { id } => {
                    let snapshot = e.graph.load();
                    let index = &snapshot.1;
                    let node_idx = index.get(&id)
                        .ok_or_else(|| ToolError::not_found("decision", &id))?;
                    let meta = &snapshot.0[*node_idx];

                    if meta.page_type != PageType::Decision {
                        return Err(ToolError::not_found("decision", &id));
                    }

                    let content = std::fs::read_to_string(&meta.path)
                        .map_err(|e| ToolError::io_error("read", meta.path.to_string_lossy(), e))?;
                    let (_fm, body) = parser::extract_frontmatter(&content);

                    Ok(json!({
                        "id": meta.id,
                        "title": meta.title,
                        "status": meta.status.as_str(),
                        "context": meta.decision_data.as_ref().map(|d| &d.context),
                        "options": meta.decision_data.as_ref().map(|d| &d.options),
                        "rationale": meta.decision_data.as_ref().map(|d| &d.rationale),
                        "outcome": meta.decision_data.as_ref().map(|d| &d.outcome),
                        "content": body,
                    }))
                }
            }
        },
    );
}
