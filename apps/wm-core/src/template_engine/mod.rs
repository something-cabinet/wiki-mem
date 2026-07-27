use serde_json::Value;
use std::fmt;

pub mod helpers;

#[derive(Debug)]
pub struct TemplateError {
    message: String,
}

impl TemplateError {
    pub fn internal(msg: impl Into<String>) -> Self {
        Self {
            message: msg.into(),
        }
    }
}

impl fmt::Display for TemplateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for TemplateError {}

pub use helpers::*;

#[derive(Debug)]
pub struct RenderResult {
    pub output: String,
    pub referenced_templates: Vec<String>,
}

pub fn render_template(
    template: &str,
    variables: &serde_json::Map<String, Value>,
    resolve_template: &dyn Fn(&str) -> Result<String, TemplateError>,
    depth: usize,
) -> Result<RenderResult, TemplateError> {
    if depth > 10 {
        return Err(TemplateError::internal(
            "Template recursion depth exceeded (max 10)",
        ));
    }

    let mut output = String::new();
    let mut remaining = template;
    let mut referenced_templates = Vec::new();

    while !remaining.is_empty() {
        let start = match remaining.find("{{") {
            Some(pos) => pos,
            None => {
                output.push_str(remaining);
                break;
            }
        };

        output.push_str(&remaining[..start]);
        remaining = &remaining[start.wrapping_add(2)..];

        let end = match remaining.find("}}") {
            Some(pos) => pos,
            None => {
                output.push_str("{{");
                output.push_str(remaining);
                break;
            }
        };

        let tag = &remaining[..end].trim();
        remaining = &remaining[end.wrapping_add(2)..];

        if tag.is_empty() {
            output.push_str("{{}}");
            continue;
        }

        let next_depth = depth.wrapping_add(1);

        if let Some(inner) = tag.strip_prefix("#if ") {
            let cond_var = inner.trim();
            let block_content = extract_block(&mut remaining, "if")?;
            let cond_val = resolve_condition(cond_var, variables);

            if is_truthy(&cond_val) {
                let result =
                    render_template(&block_content, variables, resolve_template, next_depth)?;
                output.push_str(&result.output);
                referenced_templates.extend(result.referenced_templates);
            } else {
                if let Some(else_pos) = block_content.find("{{else}}") {
                    let else_block = block_content[else_pos.wrapping_add(8)..].to_string();
                    let result =
                        render_template(&else_block, variables, resolve_template, next_depth)?;
                    output.push_str(&result.output);
                    referenced_templates.extend(result.referenced_templates);
                }
            }
        } else if let Some(inner) = tag.strip_prefix("#unless ") {
            let cond_var = inner.trim();
            let block_content = extract_block(&mut remaining, "unless")?;
            let cond_val = resolve_condition(cond_var, variables);
            if !is_truthy(&cond_val) {
                let result =
                    render_template(&block_content, variables, resolve_template, next_depth)?;
                output.push_str(&result.output);
                referenced_templates.extend(result.referenced_templates);
            }
        } else if let Some(inner) = tag.strip_prefix("#each ") {
            let list_var = inner.trim();
            let block_content = extract_block(&mut remaining, "each")?;
            let items = resolve_variable(list_var, variables);

            match items {
                Value::Array(arr) => {
                    for item in &arr {
                        let mut ctx = variables.clone();
                        ctx.insert("this".into(), item.clone());
                        if let Value::Object(map) = item {
                            for (k, v) in map {
                                ctx.insert(k.clone(), v.clone());
                            }
                        }
                        let result =
                            render_template(&block_content, &ctx, resolve_template, next_depth)?;
                        output.push_str(&result.output);
                        referenced_templates.extend(result.referenced_templates);
                    }
                }
                _ => {
                    let mut ctx = variables.clone();
                    ctx.insert("this".into(), items);
                    let result =
                        render_template(&block_content, &ctx, resolve_template, next_depth)?;
                    output.push_str(&result.output);
                    referenced_templates.extend(result.referenced_templates);
                }
            }
        } else if tag.starts_with('/') {
            return Err(TemplateError::internal(format!(
                "Unexpected closing tag '{}' with no matching opening tag",
                tag
            )));
        } else if let Some(ref_name) = tag.strip_prefix("@template/") {
            let (name, args) = parse_template_ref(ref_name);
            referenced_templates.push(name.clone());
            let tmpl_content = resolve_template(&name)?;
            let result = render_template(&tmpl_content, &args, resolve_template, next_depth)?;
            output.push_str(&result.output);
            referenced_templates.extend(result.referenced_templates);
        } else if tag.contains(' ') {
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
                output.push_str(&helpers::case_helpers::apply_helper(helper, &value_str));
            } else {
                output.push_str(&format!("{{{{{}}}}}", tag));
            }
        } else {
            let value = resolve_variable(tag, variables);
            match &value {
                Value::String(s) => output.push_str(s),
                Value::Null => {}
                other => output.push_str(&other.to_string()),
            }
        }
    }

    Ok(RenderResult {
        output,
        referenced_templates,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn noop_resolver(_name: &str) -> Result<String, TemplateError> {
        Err(TemplateError::internal(format!(
            "template not found: {_name}"
        )))
    }

    #[test]
    fn test_simple_variable() {
        let mut vars = serde_json::Map::new();
        vars.insert("name".into(), json!("World"));
        let result = render_template("Hello {{name}}!", &vars, &noop_resolver, 0).unwrap();
        assert_eq!(result.output, "Hello World!");
    }

    #[test]
    fn test_unknown_variable_empty() {
        let vars = serde_json::Map::new();
        let result = render_template("Hello {{unknown}}!", &vars, &noop_resolver, 0).unwrap();
        assert_eq!(result.output, "Hello !");
    }

    #[test]
    fn test_if_truthy() {
        let mut vars = serde_json::Map::new();
        vars.insert("show".into(), json!(true));
        let result =
            render_template("{{#if show}}visible{{/if}}", &vars, &noop_resolver, 0).unwrap();
        assert_eq!(result.output, "visible");
    }

    #[test]
    fn test_if_falsy() {
        let mut vars = serde_json::Map::new();
        vars.insert("show".into(), json!(false));
        let result =
            render_template("{{#if show}}visible{{/if}}", &vars, &noop_resolver, 0).unwrap();
        assert_eq!(result.output, "");
    }

    #[test]
    fn test_each_array() {
        let mut vars = serde_json::Map::new();
        vars.insert("items".into(), json!(["a", "b", "c"]));
        let result =
            render_template("{{#each items}}{{this}}{{/each}}", &vars, &noop_resolver, 0).unwrap();
        assert_eq!(result.output, "abc");
    }

    #[test]
    fn test_each_objects() {
        let mut vars = serde_json::Map::new();
        vars.insert(
            "items".into(),
            json!([
                {"name": "Alice"},
                {"name": "Bob"}
            ]),
        );
        let result =
            render_template("{{#each items}}{{name}}{{/each}}", &vars, &noop_resolver, 0).unwrap();
        assert_eq!(result.output, "AliceBob");
    }

    #[test]
    fn test_pascal_case_helper() {
        let mut vars = serde_json::Map::new();
        vars.insert("var".into(), json!("hello_world"));
        let result = render_template("{{pascalCase var}}", &vars, &noop_resolver, 0).unwrap();
        assert_eq!(result.output, "HelloWorld");
    }

    #[test]
    fn test_camel_case_helper() {
        let mut vars = serde_json::Map::new();
        vars.insert("var".into(), json!("hello_world"));
        let result = render_template("{{camelCase var}}", &vars, &noop_resolver, 0).unwrap();
        assert_eq!(result.output, "helloWorld");
    }

    #[test]
    fn test_kebab_case_helper() {
        let mut vars = serde_json::Map::new();
        vars.insert("var".into(), json!("helloWorld"));
        let result = render_template("{{kebabCase var}}", &vars, &noop_resolver, 0).unwrap();
        assert_eq!(result.output, "hello-world");
    }

    #[test]
    fn test_snake_case_helper() {
        let mut vars = serde_json::Map::new();
        vars.insert("var".into(), json!("helloWorld"));
        let result = render_template("{{snakeCase var}}", &vars, &noop_resolver, 0).unwrap();
        assert_eq!(result.output, "hello_world");
    }

    #[test]
    fn test_dot_notation() {
        let mut vars = serde_json::Map::new();
        vars.insert("user".into(), json!({"name": "Alice"}));
        let result = render_template("Hello {{user.name}}!", &vars, &noop_resolver, 0).unwrap();
        assert_eq!(result.output, "Hello Alice!");
    }

    #[test]
    fn test_depth_limit() {
        let vars = serde_json::Map::new();
        let nested = "{{#if true}}".repeat(15) + "deep" + &"{{/if}}".repeat(15);
        let result = render_template(&nested, &vars, &noop_resolver, 0);
        assert!(result.is_err());
    }

    #[test]
    fn test_unless() {
        let mut vars = serde_json::Map::new();
        vars.insert("hidden".into(), json!(false));
        let result = render_template(
            "{{#unless hidden}}visible{{/unless}}",
            &vars,
            &noop_resolver,
            0,
        )
        .unwrap();
        assert_eq!(result.output, "visible");
    }

    #[test]
    fn test_if_else() {
        let mut vars = serde_json::Map::new();
        vars.insert("show".into(), json!(false));
        let result =
            render_template("{{#if show}}yes{{else}}no{{/if}}", &vars, &noop_resolver, 0).unwrap();
        assert_eq!(result.output, "no");
    }

    #[test]
    fn test_render_with_all_helpers() {
        let mut vars = serde_json::Map::new();
        vars.insert("raw".into(), json!("hello_world_test"));
        let tpl = "Pascal: {{pascalCase raw}}\nCamel: {{camelCase raw}}\nKebab: {{kebabCase raw}}\nSnake: {{snakeCase raw}}\nUpper: {{upperCase raw}}\nLower: {{lowerCase raw}}";
        let result = render_template(tpl, &vars, &noop_resolver, 0).unwrap();
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

    #[test]
    fn test_template_ref_recursion_limit() {
        let vars = serde_json::Map::new();
        let result = render_template(
            "{{@template/self_ref key=val}}",
            &vars,
            &|name| {
                if name == "self_ref" {
                    Ok("{{@template/self_ref key=val}}".into())
                } else {
                    Err(TemplateError::internal(format!(
                        "template not found: {name}"
                    )))
                }
            },
            0,
        );
        assert!(result.is_err(), "deep recursion should error");
        assert!(
            result.unwrap_err().to_string().contains("recursion"),
            "error should mention recursion"
        );
    }
}
