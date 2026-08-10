use crate::engine::TemplateConfig;
use crate::error::ToolResult;
use crate::mcp::prelude::*;
use wm_constants::*;

pub use action::*;
pub use output::*;

mod action;
mod output;

/// Confine a template's storage path (create writes under `.wm/templates/`),
/// enriching the rejection with the offending template name.
fn confine_template_name(templates_dir: &std::path::Path, name: &str) -> ToolResult<std::path::PathBuf> {
    crate::shared::helpers::path_confine_helper::confine(
        templates_dir,
        std::path::Path::new(name),
    )
    .map_err(|e| {
        ToolError::invalid_params(format!(
            "{} (offending template name: {})",
            e.message,
            crate::shared::audit_sink::sanitize(name)
        ))
    })
}

pub fn register(registry: &mut ToolRegistry, engine: Arc<EngineState>) {
    registry.register_typed(
        "wm_template",
        "Manage templates: list, get, create. Supports both legacy .json templates and directory-based .wm/templates/{name}/_template.yaml templates.",
        move |input: WmTemplateAction| {
            match input {
                WmTemplateAction::List {} => handle_list(&engine),
                WmTemplateAction::Get { name } => handle_get(&engine, &name),
                WmTemplateAction::Create { name, description, content } => handle_create(&engine, &name, &description, &content),
            }
        },
    );
}

fn handle_list(engine: &Arc<EngineState>) -> Result<serde_json::Value, ToolError> {
    let root = resolve_root(engine)?;
    let templates_dir = root.join(WM_DIR).join("templates");

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

        if path.extension().and_then(|s| s.to_str()) == Some("json") {
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
                "format": "json",
            }));
        } else if path.is_dir() {
            let config_path = path.join("_template.yaml");
            if config_path.exists() {
                let config_content = match std::fs::read_to_string(&config_path) {
                    Ok(c) => c,
                    Err(_) => continue,
                };

                let config: TemplateConfig = match serde_yaml::from_str(&config_content) {
                    Ok(c) => c,
                    Err(_) => continue,
                };

                let prompt_names: Vec<&str> =
                    config.prompts.iter().map(|p| p.name.as_str()).collect();

                templates.push(serde_json::json!({
                    "name": config.name,
                    "description": config.description,
                    "prompt_count": config.prompts.len(),
                    "action_count": config.actions.len(),
                    "prompts": prompt_names,
                    "format": "directory",
                }));
            }
        }
    }

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
}

fn handle_get(engine: &Arc<EngineState>, name: &str) -> Result<serde_json::Value, ToolError> {
    let root = resolve_root(engine)?;
    let templates_dir = root.join(WM_DIR).join("templates");

    let dir_path = templates_dir.join(name).join("_template.yaml");
    if dir_path.exists() {
        let content = std::fs::read_to_string(&dir_path)
            .map_err(|e| ToolError::io_error("read", dir_path.to_string_lossy(), e))?;
        let config: TemplateConfig = serde_yaml::from_str(&content)
            .map_err(|e| ToolError::serde_error("deserialize template config", e))?;

        let template_dir = templates_dir.join(name);
        let mut hbs_files: Vec<String> = Vec::new();
        if let Ok(entries) = std::fs::read_dir(&template_dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.extension().and_then(|s| s.to_str()) == Some("hbs") {
                    if let Some(stem) = p.file_stem().and_then(|s| s.to_str()) {
                        hbs_files.push(stem.to_string());
                    }
                }
            }
        }

        return Ok(serde_json::json!({
            "name": config.name,
            "description": config.description,
            "format": "directory",
            "doc": config.doc,
            "destination": config.destination,
            "prompts": config.prompts,
            "actions": config.actions,
            "templates": hbs_files,
            "messages": config.messages,
        }));
    }

    let json_path = templates_dir.join(format!("{name}.json"));
    let content =
        std::fs::read_to_string(&json_path).map_err(|_| ToolError::not_found("template", name))?;

    let tmpl: Template = serde_json::from_str(&content)
        .map_err(|e| ToolError::serde_error("deserialize template", e))?;

    let variables = extract_variables(&tmpl.content);

    serde_json::to_value(WmTemplateGetOutput {
        name: tmpl.name,
        description: tmpl.description,
        content: tmpl.content,
        variables,
    })
    .map_err(|e| ToolError::serde_error("serialize get output", e))
}

fn handle_create(
    engine: &Arc<EngineState>,
    name: &str,
    description: &str,
    content: &str,
) -> Result<serde_json::Value, ToolError> {
    let root = resolve_root(engine)?;
    let templates_dir = root.join(WM_DIR).join("templates");

    if !templates_dir.exists() {
        std::fs::create_dir_all(&templates_dir)
            .map_err(|e| ToolError::io_error("create_dir", templates_dir.to_string_lossy(), e))?;
    }

    let confined = confine_template_name(&templates_dir, name)?;

    let json_path = templates_dir.join(format!("{name}.json"));
    let dir_path = templates_dir.join(name).join("_template.yaml");

    if json_path.exists() || dir_path.exists() {
        return Err(ToolError::internal(format!(
            "Template already exists: {name}"
        )));
    }

    let tmpl = Template {
        name: name.to_string(),
        description: description.to_string(),
        content: content.to_string(),
    };

    let json_content = serde_json::to_string_pretty(&tmpl)
        .map_err(|e| ToolError::serde_error("serialize template", e))?;

    std::fs::write(&json_path, &json_content)
        .map_err(|e| ToolError::io_error("write", json_path.to_string_lossy(), e))?;

    debug_assert!(confined.starts_with(&templates_dir));

    serde_json::to_value(WmTemplateCreateOutput {
        name: name.to_string(),
        status: "created".into(),
    })
    .map_err(|e| ToolError::serde_error("serialize create output", e))
}

fn count_variables(content: &str) -> usize {
    content.matches("{{").count()
}

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

fn resolve_root(engine: &EngineState) -> Result<std::path::PathBuf, ToolError> {
    engine
        .project_root
        .read()
        .map(|r| r.clone())
        .or_else(|_| std::env::current_dir().map_err(|e| ToolError::internal(e.to_string())))
}
