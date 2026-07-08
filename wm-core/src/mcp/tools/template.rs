use std::sync::Arc;

use crate::engine::EngineState;
use crate::error::ToolError;
use crate::mcp::handler::ToolArgs;
use crate::mcp::transport::ToolRegistry;

/// Template entry deserialized from `.wm/templates/<name>.json`
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
struct Template {
    name: String,
    description: String,
    content: String,
}

/// Register template tool handlers
pub fn register(registry: &mut ToolRegistry, engine: Arc<EngineState>) {
    // ─── wm_template.list ───────────────────────────────────────────
    let e = engine.clone();
    registry.register_with_desc(
        "template.list",
        "List all templates from .wm/templates/*.json",
        Arc::new(move |_params| {
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
        }),
    );

    // ─── wm_template.get ────────────────────────────────────────────
    let e = engine.clone();
    registry.register_with_desc(
        "template.get",
        "Get a single template by name from .wm/templates/<name>.json",
        Arc::new(move |params| {
            let args = ToolArgs::new(params);
            let name = args.require_string("name")?;

            let root = resolve_root(&e)?;
            let path = root.join(".wm").join("templates").join(format!("{}.json", name));

            let content = std::fs::read_to_string(&path).map_err(|_| {
                ToolError::not_found("template", &name)
            })?;

            let tmpl: Template = serde_json::from_str(&content)
                .map_err(|e| ToolError::serde_error("deserialize template", e))?;

            let variables = extract_variables(&tmpl.content);

            Ok(serde_json::json!({
                "name": tmpl.name,
                "description": tmpl.description,
                "content": tmpl.content,
                "variables": variables,
            }))
        }),
    );

    // ─── wm_template.run ────────────────────────────────────────────
    let e = engine.clone();
    registry.register_with_desc(
        "template.run",
        "Execute a template by replacing {{variable}} placeholders with provided values",
        Arc::new(move |params| {
            let args = ToolArgs::new(params.clone());
            let name = args.require_string("name")?;

            let root = resolve_root(&e)?;
            let path = root.join(".wm").join("templates").join(format!("{}.json", name));

            let content = std::fs::read_to_string(&path).map_err(|_| {
                ToolError::not_found("template", &name)
            })?;

            let tmpl: Template = serde_json::from_str(&content)
                .map_err(|e| ToolError::serde_error("deserialize template", e))?;

            // Parse variables from the params — they can be passed as a JSON object
            // under the "variables" key, or as a JSON string under "variables"
            let vars: serde_json::Map<String, serde_json::Value> = {
                let raw = params.get("variables");
                if let Some(serde_json::Value::Object(obj)) = raw {
                    obj.clone()
                } else if let Some(serde_json::Value::String(s)) = raw {
                    serde_json::from_str(s)
                        .map_err(|e| ToolError::internal(format!("Invalid variables JSON: {}", e)))?
                } else {
                    serde_json::Map::new()
                }
            };

            let rendered = render_template(&tmpl.content, &vars);

            Ok(serde_json::json!({
                "name": tmpl.name,
                "rendered": rendered,
            }))
        }),
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

/// Replace all {{variable}} placeholders with values from the map.
/// Unknown variables are left as-is.
fn render_template(content: &str, vars: &serde_json::Map<String, serde_json::Value>) -> String {
    let mut result = content.to_string();
    for (key, value) in vars {
        let placeholder = format!("{{{{{}}}}}", key);
        let replacement = match value {
            serde_json::Value::String(s) => s.clone(),
            other => other.to_string(),
        };
        result = result.replace(&placeholder, &replacement);
    }
    result
}

/// Resolve the project root from engine state or fallback to current directory.
fn resolve_root(engine: &EngineState) -> Result<std::path::PathBuf, ToolError> {
    engine
        .project_root
        .read()
        .map(|r| r.clone())
        .or_else(|_| Ok(std::env::current_dir().map_err(|e| ToolError::internal(e.to_string()))?))
}
