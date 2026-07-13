// ─── Template Engine — Handlebars-style Rendering ───────────
//
// Supports:
//   {{variable}}                     — simple substitution
//   {{#if var}}...{{else}}...{{/if}} — conditional blocks
//   {{#unless var}}...{{/unless}}    — inverted conditional
//   {{#each list}}...{{/each}}       — iteration
//   {{pascalCase name}}              — case helpers
//   {{camelCase name}}
//   {{kebabCase name}}
//   {{snakeCase name}}
//   {{upperCase name}}
//   {{lowerCase name}}
//   {{startCase name}}
//   {{@template/name key=val}}       — template reference resolution


use serde_json::Value;

use crate::error::ToolError;

/// Result of template rendering.
#[derive(Debug)]
pub struct RenderResult {
    pub output: String,
    pub referenced_templates: Vec<String>,
}

/// Render a template string with the given variable context.
/// Returns the rendered output and a list of referenced template names.
pub fn render(
    template: &str,
    variables: &serde_json::Map<String, Value>,
    resolve_template: &dyn Fn(&str) -> Result<String, ToolError>,
    depth: usize,
) -> Result<RenderResult, ToolError> {
    if depth > 10 {
        return Err(ToolError::internal("Template recursion depth exceeded (max 10)"));
    }

    let mut output = String::new();
    let mut remaining = template;
    let mut referenced_templates = Vec::new();

    while !remaining.is_empty() {
        // Find next {{ block
        let start = match remaining.find("{{") {
            Some(pos) => pos,
            None => {
                output.push_str(remaining);
                break;
            }
        };

        // Push text before {{
        output.push_str(&remaining[..start]);
        remaining = &remaining[start + 2..];

        // Find closing }}
        let end = match remaining.find("}}") {
            Some(pos) => pos,
            None => {
                output.push_str("{{");
                output.push_str(remaining);
                break;
            }
        };

        let tag = &remaining[..end].trim();
        remaining = &remaining[end + 2..];

        if tag.is_empty() {
            output.push_str("{{}}");
            continue;
        }

        // Determine tag type
        if let Some(inner) = tag.strip_prefix("#if ") {
            // {{#if var}}content{{/if}}
            let cond_var = inner.trim();
            let block = extract_block(&mut remaining, "if")?;
            let cond_val = resolve_condition(cond_var, variables);

            if is_truthy(&cond_val) {
                let result = render(&block, variables, resolve_template, depth + 1)?;
                output.push_str(&result.output);
                referenced_templates.extend(result.referenced_templates);
            } else {
                // Check for {{else}} in block
                if let Some(else_pos) = block.find("{{else}}") {
                    let else_block = block[else_pos + 8..].to_string();
                    let result = render(&else_block, variables, resolve_template, depth + 1)?;
                    output.push_str(&result.output);
                    referenced_templates.extend(result.referenced_templates);
                }
            }
        } else if let Some(inner) = tag.strip_prefix("#unless ") {
            let cond_var = inner.trim();
            let block = extract_block(&mut remaining, "unless")?;
            let cond_val = resolve_condition(cond_var, variables);
            if !is_truthy(&cond_val) {
                let result = render(&block, variables, resolve_template, depth + 1)?;
                output.push_str(&result.output);
                referenced_templates.extend(result.referenced_templates);
            }
        } else if let Some(inner) = tag.strip_prefix("#each ") {
            // {{#each list}}...{{/each}}
            let list_var = inner.trim();
            let block = extract_block(&mut remaining, "each")?;
            let items = resolve_variable(list_var, variables);

            match items {
                Value::Array(arr) => {
                    for item in &arr {
                        let mut ctx = variables.clone();
                        ctx.insert("this".to_string(), item.clone());
                        // Support {{name}} inside each blocks for object items
                        if let Value::Object(map) = item {
                            for (k, v) in map {
                                ctx.insert(k.clone(), v.clone());
                            }
                        }
                        let result = render(&block, &ctx, resolve_template, depth + 1)?;
                        output.push_str(&result.output);
                        referenced_templates.extend(result.referenced_templates);
                    }
                }
                _ => {
                    // Non-array each — render once with this=value
                    let mut ctx = variables.clone();
                    ctx.insert("this".to_string(), items);
                    let result = render(&block, &ctx, resolve_template, depth + 1)?;
                    output.push_str(&result.output);
                    referenced_templates.extend(result.referenced_templates);
                }
            }
        } else if tag.starts_with('/') {
            // Closing tag — should not happen at top level, but handle gracefully
            return Err(ToolError::internal(format!(
                "Unexpected closing tag '{}' with no matching opening tag",
                tag
            )));
        } else if let Some(ref_name) = tag.strip_prefix("@template/") {
            // {{@template/name key=val}}
            let (name, args) = parse_template_ref(ref_name);
            referenced_templates.push(name.clone());
            let tmpl_content = resolve_template(&name)?;
            let result = render(&tmpl_content, &args, resolve_template, depth + 1)?;
            output.push_str(&result.output);
            referenced_templates.extend(result.referenced_templates);
        } else if tag.contains(' ') {
            // Expression with helpers: {{pascalCase name}} or {{camelCase some_var}}
            let parts: Vec<&str> = tag.splitn(2, ' ').collect();
            if parts.len() == 2 {
                let helper = parts[0].trim();
                let arg_var = parts[1].trim();
                let value = resolve_variable(arg_var, variables);
                let value_str = match &value {
                    Value::String(s) => s.clone(),
                    Value::Null => String::new(),
                    other => other.to_string(),
                };
                output.push_str(&apply_helper(helper, &value_str));
            } else {
                output.push_str(&format!("{{{{{}}}}}", tag));
            }
        } else {
            // Simple {{variable}}
            let value = resolve_variable(tag, variables);
            match &value {
                Value::String(s) => output.push_str(s),
                Value::Null => {} // leave empty
                other => output.push_str(&other.to_string()),
            }
        }
    }

    Ok(RenderResult {
        output,
        referenced_templates,
    })
}

/// Extract a block between {{#tag}} and {{/tag}}.
/// Modifies `remaining` to point past the closing tag.
fn extract_block(remaining: &mut &str, tag: &str) -> Result<String, ToolError> {
    let mut depth = 1;
    let mut pos = 0;
    let bytes = remaining.as_bytes();

    while pos < remaining.len() {
        // Look for {{  }}
        if bytes[pos..].starts_with(b"{{") {
            let tag_end = match remaining[pos + 2..].find("}}") {
                Some(e) => pos + 2 + e,
                None => {
                    pos += 2;
                    continue;
                }
            };
            let inner_tag = &remaining[pos + 2..tag_end].trim();

            if let Some(close) = inner_tag.strip_prefix('/') {
                if close == tag {
                    depth -= 1;
                    if depth == 0 {
                        let block_content = remaining[..pos].to_string();
                        *remaining = &remaining[tag_end + 2..];
                        return Ok(block_content);
                    }
                }
            } else if inner_tag.starts_with(&format!("#{}", tag)) {
                depth += 1;
            }
            pos = tag_end + 2;
        } else {
            pos += 1;
        }
    }

    Err(ToolError::internal(format!(
        "Unclosed block '{}' — missing {{/{}}}",
        tag, tag
    )))
}

/// Parse a @template/name key=val key2=val2 reference.
fn parse_template_ref(input: &str) -> (String, serde_json::Map<String, Value>) {
    let parts: Vec<&str> = input.trim().splitn(2, ' ').collect();
    let name = parts[0].trim().to_string();
    let mut args = serde_json::Map::new();

    if parts.len() > 1 {
        let arg_str = parts[1];
        for pair in arg_str.split_whitespace() {
            if let Some(eq_pos) = pair.find('=') {
                let key = pair[..eq_pos].to_string();
                let val = pair[eq_pos + 1..].trim_matches('"').to_string();
                args.insert(key, Value::String(val));
            }
        }
    }

    (name, args)
}

/// Resolve a variable name from the context map.
/// Supports dot notation: person.name, item.title
fn resolve_variable(name: &str, variables: &serde_json::Map<String, Value>) -> Value {
    let parts: Vec<&str> = name.split('.').collect();
    let mut current: Option<&Value> = None;

    for (i, part) in parts.iter().enumerate() {
        if i == 0 {
            current = variables.get(*part);
        } else {
            match current {
                Some(Value::Object(map)) => current = map.get(*part),
                _ => return Value::Null,
            }
        }
    }

    current.cloned().unwrap_or(Value::Null)
}

/// Resolve a condition expression for {{#if}} and {{#unless}}.
/// Supports literal true/false, numbers, and variable names.
fn resolve_condition(expr: &str, variables: &serde_json::Map<String, Value>) -> Value {
    match expr.trim() {
        "true" => Value::Bool(true),
        "false" => Value::Bool(false),
        "null" | "nil" | "undefined" => Value::Null,
        other => {
            // Try as number
            if let Ok(n) = other.parse::<f64>() {
                Value::Number(serde_json::Number::from_f64(n).unwrap_or(serde_json::Number::from(0)))
            } else {
                // Try as variable
                resolve_variable(other, variables)
            }
        }
    }
}

/// Check if a value is "truthy" for {{#if}} blocks.
fn is_truthy(val: &Value) -> bool {
    match val {
        Value::Null => false,
        Value::Bool(b) => *b,
        Value::Number(n) => n.as_f64().map(|f| f != 0.0).unwrap_or(true),
        Value::String(s) => !s.is_empty(),
        Value::Array(a) => !a.is_empty(),
        Value::Object(_) => true,
    }
}

/// Apply a case conversion helper.
fn apply_helper(name: &str, value: &str) -> String {
    match name {
        "pascalCase" | "PascalCase" => to_pascal_case(value),
        "camelCase" => to_camel_case(value),
        "kebabCase" | "kebab-case" => to_kebab_case(value),
        "snakeCase" | "snake_case" => to_snake_case(value),
        "upperCase" | "UPPERCASE" => value.to_uppercase(),
        "lowerCase" | "lowercase" => value.to_lowercase(),
        "startCase" | "Start Case" => to_start_case(value),
        _ => value.to_string(), // unknown helper, passthrough
    }
}

fn to_pascal_case(s: &str) -> String {
    let mut result = String::new();
    let mut next_upper = true;
    for ch in s.chars() {
        if ch == '_' || ch == '-' || ch == ' ' {
            next_upper = true;
        } else if next_upper {
            result.push(ch.to_ascii_uppercase());
            next_upper = false;
        } else {
            result.push(ch);
        }
    }
    result
}

fn to_camel_case(s: &str) -> String {
    let pascal = to_pascal_case(s);
    let mut result = pascal;
    if let Some(c) = result.get_mut(..1) {
        c.make_ascii_lowercase();
    }
    result
}

fn to_kebab_case(s: &str) -> String {
    to_snake_case(s).replace('_', "-")
}

fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    for (i, ch) in s.chars().enumerate() {
        if ch.is_uppercase() && i > 0 {
            result.push('_');
            result.push(ch.to_ascii_lowercase());
        } else if ch == '-' || ch == ' ' {
            result.push('_');
        } else {
            result.push(ch.to_ascii_lowercase());
        }
    }
    result
}

fn to_start_case(s: &str) -> String {
    let mut result = String::new();
    let mut next_upper = true;
    for ch in s.chars() {
        if ch == '_' || ch == '-' || ch == ' ' {
            result.push(' ');
            next_upper = true;
        } else if next_upper {
            result.push(ch.to_ascii_uppercase());
            next_upper = false;
        } else {
            result.push(ch);
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn noop_resolver(_name: &str) -> Result<String, ToolError> {
        Err(ToolError::not_found("template", _name))
    }

    #[test]
    fn test_simple_variable() {
        let mut vars = serde_json::Map::new();
        vars.insert("name".into(), json!("World"));
        let result = render("Hello {{name}}!", &vars, &noop_resolver, 0).unwrap();
        assert_eq!(result.output, "Hello World!");
    }

    #[test]
    fn test_unknown_variable_empty() {
        let vars = serde_json::Map::new();
        let result = render("Hello {{unknown}}!", &vars, &noop_resolver, 0).unwrap();
        assert_eq!(result.output, "Hello !");
    }

    #[test]
    fn test_if_truthy() {
        let mut vars = serde_json::Map::new();
        vars.insert("show".into(), json!(true));
        let result = render("{{#if show}}visible{{/if}}", &vars, &noop_resolver, 0).unwrap();
        assert_eq!(result.output, "visible");
    }

    #[test]
    fn test_if_falsy() {
        let mut vars = serde_json::Map::new();
        vars.insert("show".into(), json!(false));
        let result = render("{{#if show}}visible{{/if}}", &vars, &noop_resolver, 0).unwrap();
        assert_eq!(result.output, "");
    }

    #[test]
    fn test_each_array() {
        let mut vars = serde_json::Map::new();
        vars.insert("items".into(), json!(["a", "b", "c"]));
        let result = render("{{#each items}}{{this}}{{/each}}", &vars, &noop_resolver, 0).unwrap();
        assert_eq!(result.output, "abc");
    }

    #[test]
    fn test_each_objects() {
        let mut vars = serde_json::Map::new();
        vars.insert("items".into(), json!([
            {"name": "Alice"},
            {"name": "Bob"}
        ]));
        let result = render("{{#each items}}{{name}}{{/each}}", &vars, &noop_resolver, 0).unwrap();
        assert_eq!(result.output, "AliceBob");
    }

    #[test]
    fn test_pascal_case_helper() {
        let mut vars = serde_json::Map::new();
        vars.insert("var".into(), json!("hello_world"));
        let result = render("{{pascalCase var}}", &vars, &noop_resolver, 0).unwrap();
        assert_eq!(result.output, "HelloWorld");
    }

    #[test]
    fn test_camel_case_helper() {
        let mut vars = serde_json::Map::new();
        vars.insert("var".into(), json!("hello_world"));
        let result = render("{{camelCase var}}", &vars, &noop_resolver, 0).unwrap();
        assert_eq!(result.output, "helloWorld");
    }

    #[test]
    fn test_kebab_case_helper() {
        let mut vars = serde_json::Map::new();
        vars.insert("var".into(), json!("helloWorld"));
        let result = render("{{kebabCase var}}", &vars, &noop_resolver, 0).unwrap();
        assert_eq!(result.output, "hello-world");
    }

    #[test]
    fn test_snake_case_helper() {
        let mut vars = serde_json::Map::new();
        vars.insert("var".into(), json!("helloWorld"));
        let result = render("{{snakeCase var}}", &vars, &noop_resolver, 0).unwrap();
        assert_eq!(result.output, "hello_world");
    }

    #[test]
    fn test_dot_notation() {
        let mut vars = serde_json::Map::new();
        vars.insert("user".into(), json!({"name": "Alice"}));
        let result = render("Hello {{user.name}}!", &vars, &noop_resolver, 0).unwrap();
        assert_eq!(result.output, "Hello Alice!");
    }

    #[test]
    fn test_depth_limit() {
        let vars = serde_json::Map::new();
        // 15 nested if-blocks should exceed depth limit of 10
        let nested = "{{#if true}}".repeat(15) + "deep" + &"{{/if}}".repeat(15);
        let result = render(&nested, &vars, &noop_resolver, 0);
        assert!(result.is_err(), "Expected depth limit error, got: {:?}", result);
    }

    #[test]
    fn test_unless() {
        let mut vars = serde_json::Map::new();
        vars.insert("hidden".into(), json!(false));
        let result = render("{{#unless hidden}}visible{{/unless}}", &vars, &noop_resolver, 0).unwrap();
        assert_eq!(result.output, "visible");
    }

    #[test]
    fn test_if_else() {
        let mut vars = serde_json::Map::new();
        vars.insert("show".into(), json!(false));
        let result = render("{{#if show}}yes{{else}}no{{/if}}", &vars, &noop_resolver, 0).unwrap();
        assert_eq!(result.output, "no");
    }

    #[test]
    fn test_render_with_all_helpers() {
        let mut vars = serde_json::Map::new();
        vars.insert("raw".into(), json!("hello_world_test"));
        let tpl = "Pascal: {{pascalCase raw}}\nCamel: {{camelCase raw}}\nKebab: {{kebabCase raw}}\nSnake: {{snakeCase raw}}\nUpper: {{upperCase raw}}\nLower: {{lowerCase raw}}";
        let result = render(tpl, &vars, &noop_resolver, 0).unwrap();
        assert!(result.output.contains("Pascal: HelloWorldTest"));
        assert!(result.output.contains("Camel: helloWorldTest"));
        assert!(result.output.contains("Kebab: hello-world-test"));
        assert!(result.output.contains("Snake: hello_world_test"));
    }

    #[test]
    fn test_template_ref_parsing() {
        let (name, args) = parse_template_ref("my-template key1=val1 key2=val2");
        assert_eq!(name, "my-template");
        assert_eq!(args.get("key1").and_then(|v| v.as_str()), Some("val1"));
        assert_eq!(args.get("key2").and_then(|v| v.as_str()), Some("val2"));
    }
}
