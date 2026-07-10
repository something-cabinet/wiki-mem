use std::sync::Arc;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use crate::engine::EngineState;
use crate::error::ToolError;
use crate::mcp::transport::ToolRegistry;
use crate::mcp::typed::TypedRegister;

/// Template entry deserialized from `.wm/templates/<name>.json`
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
struct Template {
    name: String,
    description: String,
    content: String,
}

// ─── Input / Output types ───────────────────────────────────

#[derive(Deserialize, JsonSchema)]
struct WmTemplateGetInput {
    #[schemars(description = "Template name")]
    name: String,
}

#[derive(Serialize)]
struct WmTemplateGetOutput {
    name: String,
    description: String,
    content: String,
    variables: Vec<String>,
}

#[derive(Deserialize, JsonSchema)]
struct WmTemplateCreateInput {
    #[schemars(description = "Template name")]
    name: String,
    #[schemars(description = "Template description")]
    description: String,
    #[schemars(description = "Template content with {{variable}} placeholders")]
    content: String,
}

#[derive(Serialize)]
struct WmTemplateCreateOutput {
    name: String,
    status: String,
}

#[derive(Deserialize, JsonSchema)]
struct WmTemplateRunInput {
    #[schemars(description = "Template name")]
    name: String,
    #[schemars(description = "Variable values keyed by variable name")]
    variables: serde_json::Map<String, serde_json::Value>,
}

#[derive(Serialize)]
struct WmTemplateRunOutput {
    name: String,
    rendered: String,
}

/// Register template tool handlers
pub fn register(registry: &mut ToolRegistry, engine: Arc<EngineState>) {
    // ─── wm_template.list ───────────────────────────────────────────
    let e = engine.clone();
    registry.register_read(
        "wm_template.list",
        "List all templates from .wm/templates/*.json",
        move |_input: EmptyInput| {
            let root = resolve_root(&e)?;
            let templates_dir = root.join(".wm").join("templates");

            if !templates_dir.exists() || !templates_dir.is_dir() {
                return Ok(serde_json::json!({
                    "templates": [],
                    "total": 0,
                    "note": ".wm/templates/ not found"
                }));
            }

            let dir_entries = match std::fs::read_dir(&templates_dir) {
                Ok(entries) => entries,
                Err(e) => {
                    return Err(ToolError::io_error(
                        "read_dir",
                        templates_dir.to_string_lossy(),
                        e,
                    ))
                }
            };

            let mut templates: Vec<serde_json::Value> = Vec::new();

            for entry in dir_entries {
                let entry = match entry {
                    Ok(e) => e,
                    Err(_) => continue,
                };
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) != Some("json") {
                    continue;
                }

                let content = match std::fs::read_to_string(&path) {
                    Ok(c) => c,
                    Err(_) => continue,
                };

                let tmpl: Template = match serde_json::from_str(&content) {
                    Ok(t) => t,
                    Err(_) => continue,
                };

                let var_count = count_variables(&tmpl.content);

                templates.push(serde_json::json!({
                    "name": tmpl.name,
                    "description": tmpl.description,
                    "variable_count": var_count,
                }));
            }

            // Sort by name for stable ordering
            templates.sort_by(|a, b| {
                a.get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .cmp(b.get("name").and_then(|v| v.as_str()).unwrap_or(""))
            });

            Ok(serde_json::json!({
                "templates": templates,
                "total": templates.len(),
            }))
        },
    );

    // ─── wm_template.get ────────────────────────────────────────────
    let e = engine.clone();
    registry.register_read(
        "wm_template.get",
        "Get a single template by name from .wm/templates/<name>.json",
        move |input: WmTemplateGetInput| {
            let name = input.name;

            let root = resolve_root(&e)?;
            let path = root.join(".wm").join("templates").join(format!("{}.json", name));

            let content = std::fs::read_to_string(&path).map_err(|_| {
                ToolError::not_found("template", &name)
            })?;

            let tmpl: Template = serde_json::from_str(&content)
                .map_err(|e| ToolError::serde_error("deserialize template", e))?;

            let variables = extract_variables(&tmpl.content);

            Ok(WmTemplateGetOutput {
                name: tmpl.name,
                description: tmpl.description,
                content: tmpl.content,
                variables,
            })
        },
    );

    // ─── wm_template.create ────────────────────────────────────────
    let e = engine.clone();
    registry.register_write(
        "wm_template.create",
        "Create a new template in .wm/templates/<name>.json",
        move |input: WmTemplateCreateInput| {
            let name = input.name;
            let description = input.description;
            let content = input.content;

            let root = resolve_root(&e)?;
            let templates_dir = root.join(".wm").join("templates");

            // Create templates directory if it doesn't exist
            if !templates_dir.exists() {
                std::fs::create_dir_all(&templates_dir)
                    .map_err(|e| ToolError::io_error("create_dir", templates_dir.to_string_lossy(), e))?;
            }

            let path = templates_dir.join(format!("{}.json", name));

            if path.exists() {
                return Err(ToolError::internal(format!("Template already exists: {}", name)));
            }

            let tmpl = Template {
                name: name.clone(),
                description,
                content,
            };

            let json_content = serde_json::to_string_pretty(&tmpl)
                .map_err(|e| ToolError::serde_error("serialize template", e))?;

            std::fs::write(&path, &json_content)
                .map_err(|e| ToolError::io_error("write", path.to_string_lossy(), e))?;

            Ok(WmTemplateCreateOutput {
                name,
                status: "created".to_string(),
            })
        },
    );

    // ─── wm_template.run ────────────────────────────────────────────
    let e = engine.clone();
    registry.register_write(
        "wm_template.run",
        "Render a template with variable substitution. Supports {{variable}}, {{#if}}/{{#each}} blocks, case helpers (pascalCase, camelCase, kebabCase, snakeCase, upperCase, lowerCase), and @template references.",
        move |input: WmTemplateRunInput| {
            let name = input.name;
            let vars = input.variables;

            let root = resolve_root(&e)?;
            let path = root.join(".wm").join("templates").join(format!("{}.json", name));

            let content = std::fs::read_to_string(&path).map_err(|_| {
                ToolError::not_found("template", &name)
            })?;

            let tmpl: Template = serde_json::from_str(&content)
                .map_err(|e| ToolError::serde_error("deserialize template", e))?;

            let root_for_resolver = root.clone();
            let resolve_tmpl = |ref_name: &str| -> Result<String, ToolError> {
                let ref_path = root_for_resolver.join(".wm").join("templates").join(format!("{}.json", ref_name));
                let ref_content = std::fs::read_to_string(&ref_path)
                    .map_err(|_| ToolError::not_found("template", ref_name))?;
                let t: Template = serde_json::from_str(&ref_content)
                    .map_err(|e| ToolError::serde_error("deserialize template", e))?;
                Ok(t.content)
            };

            let result = crate::template_engine::render(&tmpl.content, &vars, &resolve_tmpl, 0)
                .map_err(|e| ToolError::internal(format!("Template render error: {}", e)))?;

            Ok(WmTemplateRunOutput {
                name: tmpl.name,
                rendered: result.output,
            })
        },
    );
}

/// Count {{variable}} placeholders in template content
fn count_variables(content: &str) -> usize {
    content.matches("{{").count()
}

/// Extract unique variable names from {{variable}} placeholders
fn extract_variables(content: &str) -> Vec<String> {
    use std::collections::BTreeSet;

    let mut vars = BTreeSet::new();
    let mut remaining = content;
    while let Some(start) = remaining.find("{{") {
        let after_start = &remaining[start + 2..];
        if let Some(end) = after_start.find("}}") {
            let var_name = after_start[..end].trim().to_string();
            if !var_name.is_empty() {
                vars.insert(var_name);
            }
            remaining = &after_start[end + 2..];
        } else {
            break;
        }
    }
    vars.into_iter().collect()
}

/// Resolve the project root from engine state or fallback to current directory.
fn resolve_root(engine: &EngineState) -> Result<std::path::PathBuf, ToolError> {
    engine
        .project_root
        .read()
        .map(|r| r.clone())
        .or_else(|_| Ok(std::env::current_dir().map_err(|e| ToolError::internal(e.to_string()))?))
}

/// Empty input for tools that take no parameters.
#[derive(Deserialize, JsonSchema)]
struct EmptyInput {}
