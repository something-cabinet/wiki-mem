use crate::mcp::prelude::*;
use crate::engine::TemplateConfig;
use crate::template_engine::{render_template, TemplateError};
use walkdir::WalkDir;

pub use action::*;
pub use output::*;

mod action;
mod output;

pub fn register(registry: &mut ToolRegistry, engine: Arc<EngineState>) {
    registry.register_typed(
        "wm_template",
        "Manage templates: list, get, create, run. Supports both legacy .json templates and directory-based .wm/templates/{name}/_template.yaml templates.",
        move |input: WmTemplateAction| {
            match input {
                WmTemplateAction::List {} => handle_list(&engine),
                WmTemplateAction::Get { name } => handle_get(&engine, &name),
                WmTemplateAction::Create { name, description, content } => handle_create(&engine, &name, &description, &content),
                WmTemplateAction::Run { name, variables } => handle_run(&engine, &name, variables),
            }
        },
    );
}


fn handle_list(engine: &Arc<EngineState>) -> Result<serde_json::Value, ToolError> {
    let root = resolve_root(engine)?;
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

                let prompt_names: Vec<&str> = config.prompts.iter().map(|p| p.name.as_str()).collect();

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
    let templates_dir = root.join(".wm").join("templates");

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
    let content = std::fs::read_to_string(&json_path).map_err(|_| {
        ToolError::not_found("template", name)
    })?;

    let tmpl: Template = serde_json::from_str(&content)
        .map_err(|e| ToolError::serde_error("deserialize template", e))?;

    let variables = extract_variables(&tmpl.content);

    Ok(serde_json::to_value(WmTemplateGetOutput {
        name: tmpl.name,
        description: tmpl.description,
        content: tmpl.content,
        variables,
    }).map_err(|e| ToolError::serde_error("serialize get output", e))?)
}

fn handle_create(engine: &Arc<EngineState>, name: &str, description: &str, content: &str) -> Result<serde_json::Value, ToolError> {
    let root = resolve_root(engine)?;
    let templates_dir = root.join(".wm").join("templates");

    if !templates_dir.exists() {
        std::fs::create_dir_all(&templates_dir)
            .map_err(|e| ToolError::io_error("create_dir", templates_dir.to_string_lossy(), e))?;
    }

    let json_path = templates_dir.join(format!("{name}.json"));
    let dir_path = templates_dir.join(name).join("_template.yaml");

    if json_path.exists() || dir_path.exists() {
        return Err(ToolError::internal(format!("Template already exists: {name}")));
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

    Ok(serde_json::to_value(WmTemplateCreateOutput {
        name: name.to_string(),
        status: "created".to_string(),
    }).map_err(|e| ToolError::serde_error("serialize create output", e))?)
}

fn handle_run(engine: &Arc<EngineState>, name: &str, variables: Option<std::collections::HashMap<String, String>>) -> Result<serde_json::Value, ToolError> {
    let root = resolve_root(engine)?;
    let templates_dir = root.join(".wm").join("templates");

    let dir_path = templates_dir.join(name).join("_template.yaml");
    if dir_path.exists() {
        return run_directory_template(engine, &templates_dir, name, variables);
    }

    run_json_template(&templates_dir, name, variables)
}


fn run_directory_template(
    engine: &Arc<EngineState>,
    templates_dir: &std::path::Path,
    name: &str,
    variables: Option<std::collections::HashMap<String, String>>,
) -> Result<serde_json::Value, ToolError> {
    let config_path = templates_dir.join(name).join("_template.yaml");
    let content = std::fs::read_to_string(&config_path)
        .map_err(|e| ToolError::io_error("read", config_path.to_string_lossy(), e))?;

    let config: TemplateConfig = serde_yaml::from_str(&content)
        .map_err(|e| ToolError::serde_error("deserialize template config", e))?;

    let template_dir = templates_dir.join(name);

    let user_vars = variables.unwrap_or_default();
    let mut ctx: serde_json::Map<String, serde_json::Value> = serde_json::Map::new();

    for prompt in &config.prompts {
        let val = if let Some(user_val) = user_vars.get(&prompt.name) {
            serde_json::Value::String(user_val.clone())
        } else if let Some(initial) = &prompt.initial {
            initial.clone()
        } else {
            serde_json::Value::Null
        };
        ctx.insert(prompt.name.clone(), val);
    }

    for (k, v) in &user_vars {
        if !ctx.contains_key(k) {
            ctx.insert(k.clone(), serde_json::Value::String(v.clone()));
        }
    }

    let td = template_dir.clone();
    let resolve_tmpl = move |ref_name: &str| -> Result<String, TemplateError> {
        let hbs_path = td.join(format!("{ref_name}.hbs"));
        if hbs_path.exists() {
            return std::fs::read_to_string(&hbs_path)
                .map_err(|e| TemplateError::internal(format!("read {}: {}", hbs_path.display(), e)));
        }
        let json_path = td.join(format!("{ref_name}.json"));
        if json_path.exists() {
            return std::fs::read_to_string(&json_path)
                .map_err(|e| TemplateError::internal(format!("read {}: {}", json_path.display(), e)));
        }
        Err(TemplateError::internal(format!("Template reference not found: {ref_name}")))
    };

    let destination = config.destination.as_deref().unwrap_or(".");

    let mut results: Vec<serde_json::Value> = Vec::new();
    let dest_path = resolve_root(engine)?.join(destination);

    for action in &config.actions {
                let action_result = execute_action(action, &template_dir, &dest_path, &ctx, &resolve_tmpl)?;
        results.push(action_result);
    }

    Ok(serde_json::json!({
        "name": config.name,
        "rendered": true,
        "action_count": config.actions.len(),
        "results": results,
        "description": config.description,
    }))
}

fn execute_action(
    action: &crate::engine::TemplateAction,
    template_dir: &std::path::Path,
    dest_dir: &std::path::Path,
    ctx: &serde_json::Map<String, serde_json::Value>,
    resolve_tmpl: &dyn Fn(&str) -> Result<String, TemplateError>,
) -> Result<serde_json::Value, ToolError> {
    if let Some(ref when_expr) = action.when {
        let template_str = if when_expr.contains("{{") {
            when_expr.clone()
        } else {
            format!("{{{{{}}}}}", when_expr.trim())
        };
        let rendered_when = render_template(
            &template_str, ctx, resolve_tmpl, 0,
        )
        .map_err(|e| ToolError::internal(format!("When condition render error: {e}")))?;
        let trimmed = rendered_when.output.trim().to_lowercase();
        let is_truthy = !trimmed.is_empty()
            && trimmed != "false"
            && trimmed != "no"
            && trimmed != "0"
            && trimmed != "null";
        if !is_truthy {
            return Ok(serde_json::json!({
                "action": action.r#type,
                "status": "skipped",
                "reason": "when condition not met",
            }));
        }
    }

    match action.r#type.as_str() {
        "add" => {
            let tmpl_name = action.template.as_deref().unwrap_or("default");
            let tmpl_content = resolve_tmpl(tmpl_name)
                .map_err(|e| ToolError::internal(e.to_string()))?;

            let rendered = render_template(&tmpl_content, ctx, resolve_tmpl, 0)
                .map_err(|e| ToolError::internal(format!("Template render error: {e}")))?;

            let output_path = render_path(&action.path, ctx);
            let full_path = dest_dir.join(&output_path);

            if action.skip_if_exists.unwrap_or(false) && full_path.exists() {
                return Ok(serde_json::json!({
                    "action": "add",
                    "path": output_path,
                    "status": "skipped",
                    "reason": "file exists",
                }));
            }

            if let Some(parent) = full_path.parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| ToolError::io_error("create_dir", parent.to_string_lossy(), e))?;
            }

            std::fs::write(&full_path, &rendered.output)
                .map_err(|e| ToolError::io_error("write", full_path.to_string_lossy(), e))?;

            Ok(serde_json::json!({
                "action": "add",
                "path": output_path,
                "status": "created",
                "size": rendered.output.len(),
            }))
        }
        "addMany" => {
            let source_dir_name = action.source.as_deref().unwrap_or(".");
            let source_dir = template_dir.join(source_dir_name);

            if !source_dir.exists() || !source_dir.is_dir() {
                return Ok(serde_json::json!({
                    "action": "addMany",
                    "items": [],
                    "total": 0,
                    "note": format!("Source directory not found: {}", source_dir.display()),
                }));
            }

            let dest_dir_str = render_path(&action.path, ctx);
            let base_dest = if dest_dir_str.is_empty() {
                dest_dir.to_path_buf()
            } else {
                dest_dir.join(&dest_dir_str)
            };

            let mut items: Vec<serde_json::Value> = Vec::new();

            for entry in WalkDir::new(&source_dir)
                .min_depth(1)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                let path = entry.path();
                if !path.is_file() {
                    continue;
                }
                if path.extension().and_then(|s| s.to_str()) != Some("hbs") {
                    continue;
                }

                let tmpl_content = std::fs::read_to_string(path)
                    .map_err(|e| ToolError::io_error("read", path.to_string_lossy(), e))?;

                let rendered = render_template(&tmpl_content, ctx, resolve_tmpl, 0)
                    .map_err(|e| ToolError::internal(format!("Template render error: {e}")))?;

                let relative = path.strip_prefix(&source_dir)
                    .map_err(|_| ToolError::internal("Failed to strip source directory prefix"))?;

                let relative_stem = relative.with_extension("");

                let output_path = base_dest.join(&relative_stem);

                if let Some(parent) = output_path.parent() {
                    std::fs::create_dir_all(parent)
                        .map_err(|e| ToolError::io_error("create_dir", parent.to_string_lossy(), e))?;
                }

                std::fs::write(&output_path, &rendered.output)
                    .map_err(|e| ToolError::io_error("write", output_path.to_string_lossy(), e))?;

                items.push(serde_json::json!({
                    "path": output_path.to_string_lossy(),
                    "status": "created",
                    "size": rendered.output.len(),
                }));
            }

            Ok(serde_json::json!({
                "action": "addMany",
                "items": items,
                "total": items.len(),
            }))
        }
        "append" => {
            let source = action.source.as_deref().unwrap_or("default");
            let tmpl_content = resolve_tmpl(source)
                .map_err(|e| ToolError::internal(e.to_string()))?;

            let rendered = render_template(&tmpl_content, ctx, resolve_tmpl, 0)
                .map_err(|e| ToolError::internal(format!("Template render error: {e}")))?;

            let output_path = render_path(&action.path, ctx);
            let full_path = dest_dir.join(&output_path);

            let mut existing = String::new();
            if full_path.exists() {
                existing = std::fs::read_to_string(&full_path)
                    .map_err(|e| ToolError::io_error("read", full_path.to_string_lossy(), e))?;
            } else {
                if let Some(parent) = full_path.parent() {
                    std::fs::create_dir_all(parent)
                        .map_err(|e| ToolError::io_error("create_dir", parent.to_string_lossy(), e))?;
                }
            }

            let new_content = if existing.is_empty() {
                rendered.output.clone()
            } else {
                format!("{existing}\n{}", rendered.output)
            };

            std::fs::write(&full_path, &new_content)
                .map_err(|e| ToolError::io_error("write", full_path.to_string_lossy(), e))?;

            Ok(serde_json::json!({
                "action": "append",
                "path": output_path,
                "status": "appended",
                "size": rendered.output.len(),
            }))
        }
        "modify" => {
            let source = action.source.as_deref().unwrap_or("default");
            let tmpl_content = resolve_tmpl(source)
                .map_err(|e| ToolError::internal(e.to_string()))?;

            let rendered = render_template(&tmpl_content, ctx, resolve_tmpl, 0)
                .map_err(|e| ToolError::internal(format!("Template render error: {e}")))?;

            let output_path = render_path(&action.path, ctx);
            let full_path = dest_dir.join(&output_path);

            if !full_path.exists() {
                return Err(ToolError::not_found("file", &output_path));
            }

            let existing = std::fs::read_to_string(&full_path)
                .map_err(|e| ToolError::io_error("read", full_path.to_string_lossy(), e))?;

            let new_content = if let Some(ref insert_after) = action.insert_after {
                if let Some(pos) = existing.find(insert_after) {
                    let insert_pos = pos + insert_after.len();
                    let before = &existing[..insert_pos];
                    let after = &existing[insert_pos..];
                    format!("{before}\n{}\n{after}", rendered.output.trim())
                } else {
                    format!("{existing}\n{}", rendered.output)
                }
            } else {
                rendered.output.clone()
            };

            std::fs::write(&full_path, &new_content)
                .map_err(|e| ToolError::io_error("write", full_path.to_string_lossy(), e))?;

            Ok(serde_json::json!({
                "action": "modify",
                "path": output_path,
                "status": "modified",
                "size": rendered.output.len(),
            }))
        }
        other => Err(ToolError::internal(format!("Unknown action type: {other}"))),
    }
}

fn render_path(template: &str, ctx: &serde_json::Map<String, serde_json::Value>) -> String {
    let mut result = String::new();
    let mut remaining = template;

    while let Some(start) = remaining.find("{{") {
        result.push_str(&remaining[..start]);
        let after_start = &remaining[start + 2..];
        if let Some(end) = after_start.find("}}") {
            let var_name = after_start[..end].trim();
            let value = match ctx.get(var_name) {
                Some(serde_json::Value::String(s)) => s.clone(),
                Some(serde_json::Value::Null) => String::new(),
                Some(other) => other.to_string(),
                None => String::new(),
            };
            result.push_str(&value);
            remaining = &after_start[end + 2..];
        } else {
            result.push_str(&remaining[start..]);
            break;
        }
    }

    result.push_str(remaining);
    result
}


fn run_json_template(
    templates_dir: &std::path::Path,
    name: &str,
    variables: Option<std::collections::HashMap<String, String>>,
) -> Result<serde_json::Value, ToolError> {
    let path = templates_dir.join(format!("{name}.json"));

    let content = std::fs::read_to_string(&path).map_err(|_| {
        ToolError::not_found("template", name)
    })?;

    let tmpl: Template = serde_json::from_str(&content)
        .map_err(|e| ToolError::serde_error("deserialize template", e))?;

    let vars: serde_json::Map<String, serde_json::Value> = variables
        .unwrap_or_default()
        .into_iter()
        .map(|(k, v)| (k, serde_json::Value::String(v)))
        .collect();

    let td = templates_dir.to_path_buf();
    let resolve_tmpl = |ref_name: &str| -> Result<String, TemplateError> {
        let hbs_path = td.join(format!("{ref_name}.hbs"));
        if hbs_path.exists() {
            return std::fs::read_to_string(&hbs_path)
                .map_err(|e| TemplateError::internal(format!("read {}: {}", hbs_path.display(), e)));
        }
        let ref_path = td.join(format!("{ref_name}.json"));
        let ref_content = std::fs::read_to_string(&ref_path)
            .map_err(|_| TemplateError::internal(format!("Template not found: {ref_name}")))?;
        let t: Template = serde_json::from_str(&ref_content)
            .map_err(|e| TemplateError::internal(format!("deserialize template: {e}")))?;
        Ok(t.content)
    };

    let result = render_template(&tmpl.content, &vars, &resolve_tmpl, 0)
        .map_err(|e| ToolError::internal(format!("Template render error: {e}")))?;

    Ok(serde_json::to_value(WmTemplateRunOutput {
        name: tmpl.name,
        rendered: result.output,
    }).map_err(|e| ToolError::serde_error("serialize run output", e))?)
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
