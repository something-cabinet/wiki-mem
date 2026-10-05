use super::frontmatter_value::FrontmatterValue;

const FRONTMATTER_INDENT: &str = "  ";

pub fn build_frontmatter(fields: &[(&'static str, FrontmatterValue)]) -> String {
    let mut out = String::new();
    push_frontmatter_fields(&mut out, fields, 0);
    out
}

fn push_frontmatter_fields(
    out: &mut String,
    fields: &[(&'static str, FrontmatterValue)],
    depth: usize,
) {
    let indent = FRONTMATTER_INDENT.repeat(depth);
    for (key, value) in fields {
        if let FrontmatterValue::Nested(children) = value {
            out.push_str(&format!("{indent}{key}:\n"));
            push_frontmatter_fields(out, children, depth.wrapping_add(1));
            continue;
        }
        let rendered = render_frontmatter_scalar(value);
        out.push_str(&format!("{indent}{key}: {rendered}\n"));
    }
}

fn render_frontmatter_scalar(value: &FrontmatterValue) -> String {
    match value {
        FrontmatterValue::Scalar(s) => yaml_scalar(s),
        FrontmatterValue::Id(s) => yaml_quote(s),
        FrontmatterValue::Int(n) => n.to_string(),
        FrontmatterValue::List(items) => {
            let rendered = items
                .iter()
                .map(|item| yaml_scalar(item))
                .collect::<Vec<_>>()
                .join(", ");
            format!("[{rendered}]")
        }
        FrontmatterValue::Nested(_) => String::new(),
    }
}

pub fn yaml_scalar(value: &str) -> String {
    let rendered = serde_yaml::to_string(&serde_yaml::Value::String(value.to_string()))
        .unwrap_or_else(|_| value.to_string());
    rendered.trim_end().to_string()
}

pub fn yaml_quote(value: &str) -> String {
    let escaped = value.replace('\\', "\\\\").replace('"', "\\\"");
    format!("\"{}\"", escaped)
}

pub fn parse_yaml_mut<F>(yaml: &str, f: F) -> String
where
    F: FnOnce(&mut serde_yaml::Mapping),
{
    let trimmed = yaml.trim();
    if trimmed.is_empty() {
        let mut map = serde_yaml::Mapping::new();
        f(&mut map);
        if map.is_empty() {
            return String::new();
        }
        return serde_yaml::to_string(&serde_yaml::Value::Mapping(map)).unwrap_or_default();
    }
    match serde_yaml::from_str::<serde_yaml::Value>(yaml) {
        Ok(serde_yaml::Value::Mapping(mut map)) => {
            f(&mut map);
            let value = serde_yaml::Value::Mapping(map);
            let rendered = serde_yaml::to_string(&value).unwrap_or_else(|_| yaml.to_string());
            if rendered.trim() == "{}" {
                String::new()
            } else {
                rendered
            }
        }
        _ => yaml.to_string(),
    }
}

pub fn extract_yaml_string_value(yaml: &str, key: &str) -> String {
    let value: serde_yaml::Value = serde_yaml::from_str(yaml).unwrap_or(serde_yaml::Value::Null);
    match value {
        serde_yaml::Value::Mapping(ref map) => {
            let k = serde_yaml::Value::String(key.to_string());
            map.get(&k)
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .unwrap_or_default()
        }
        _ => String::new(),
    }
}

fn is_top_level_key(line: &str, key: &str) -> bool {
    if line.starts_with(' ') || line.starts_with('\t') {
        return false;
    }
    match line.strip_prefix(key) {
        Some(rest) => rest.trim_start().starts_with(':'),
        None => false,
    }
}

pub fn set_yaml_field(yaml: &str, key: &str, value: &str) -> String {
    let rendered = yaml_scalar(value);
    let rendered_lines: Vec<&str> = rendered.lines().collect();

    let mut out: Vec<String> = Vec::new();
    let mut replaced = false;
    let mut iter = yaml.lines().peekable();

    while let Some(line) = iter.next() {
        if !replaced && is_top_level_key(line, key) {
            for (i, rl) in rendered_lines.iter().enumerate() {
                if i == 0 {
                    out.push(format!("{}: {}", key, rl));
                } else {
                    out.push(rl.to_string());
                }
            }
            replaced = true;
            while let Some(next) = iter.peek() {
                if next.starts_with(' ') || next.starts_with('\t') {
                    iter.next();
                } else {
                    break;
                }
            }
        } else {
            out.push(line.to_string());
        }
    }

    if !replaced {
        for (i, rl) in rendered_lines.iter().enumerate() {
            if i == 0 {
                out.push(format!("{}: {}", key, rl));
            } else {
                out.push(rl.to_string());
            }
        }
    }

    let mut result = out.join("\n");
    if !result.is_empty() && !result.ends_with('\n') {
        result.push('\n');
    }
    result
}

pub fn ac_set_checked(yaml: &str, index: usize, checked: bool) -> String {
    if index == 0 {
        return yaml.to_string();
    }

    let mut out: Vec<String> = Vec::new();
    let mut in_ac = false;
    let mut item_idx = 0usize;
    let mut in_target_item = false;
    let mut target_indent: Option<usize> = None;
    let mut inserted = false;

    let flush_pending =
        |out: &mut Vec<String>, indent: Option<usize>, inserted: &mut bool, checked: bool| {
            if !*inserted {
                if let Some(i) = indent {
                    out.push(format!("{}checked: {}", " ".repeat(i + 2), checked));
                }
                *inserted = true;
            }
        };

    for line in yaml.lines() {
        if !in_ac {
            out.push(line.to_string());
            if is_top_level_key(line, "acceptance_criteria") {
                in_ac = true;
            }
            continue;
        }

        let trimmed = line.trim_start();
        let is_item = trimmed.starts_with('-');
        let is_toplevel = !line.starts_with(' ') && !line.starts_with('\t') && !is_item;

        if is_toplevel {
            flush_pending(&mut out, target_indent, &mut inserted, checked);
            in_ac = false;
            out.push(line.to_string());
            continue;
        }

        if is_item {
            flush_pending(&mut out, target_indent, &mut inserted, checked);
            item_idx += 1;
            out.push(line.to_string());
            in_target_item = item_idx == index;
            target_indent = if in_target_item {
                Some(line.len() - line.trim_start().len())
            } else {
                None
            };
            continue;
        }

        if in_target_item && !inserted {
            if trimmed.starts_with("checked:") {
                let cur_indent = line.len() - line.trim_start().len();
                out.push(format!("{}checked: {}", " ".repeat(cur_indent), checked));
                inserted = true;
            } else {
                out.push(line.to_string());
            }
            continue;
        }
        out.push(line.to_string());
    }

    flush_pending(&mut out, target_indent, &mut inserted, checked);

    let mut result = out.join("\n");
    if !result.is_empty() && !result.ends_with('\n') {
        result.push('\n');
    }
    result
}

pub fn remove_yaml_block(yaml: &str, key: &str) -> String {
    let mut out: Vec<String> = Vec::new();
    let mut skipping = false;
    for line in yaml.lines() {
        if skipping {
            if line.starts_with(' ') || line.starts_with('\t') {
                continue;
            }
            skipping = false;
        }
        if is_top_level_key(line, key) {
            skipping = true;
            continue;
        }
        out.push(line.to_string());
    }
    let mut result = out.join("\n");
    if !result.is_empty() && !result.ends_with('\n') {
        result.push('\n');
    }
    result
}

fn render_yaml_value(value: &serde_json::Value) -> String {
    let json_str = serde_json::to_string(value).unwrap_or_default();
    let yaml_val: serde_yaml::Value =
        serde_yaml::from_str(&json_str).unwrap_or(serde_yaml::Value::Null);
    serde_yaml::to_string(&yaml_val)
        .unwrap_or_default()
        .trim_end()
        .to_string()
}

pub fn set_yaml_value_field(yaml: &str, key: &str, value: &serde_json::Value) -> String {
    let rendered = render_yaml_value(value);
    let rendered_lines: Vec<&str> = rendered.lines().collect();

    let block = matches!(value, serde_json::Value::Array(a) if !a.is_empty())
        || matches!(value, serde_json::Value::Object(o) if !o.is_empty());
    let replacement: Vec<String> = if block {
        let mut lines = vec![format!("{}:", key)];
        for l in &rendered_lines {
            if l.is_empty() {
                lines.push(String::new());
            } else {
                lines.push(format!("  {}", l));
            }
        }
        lines
    } else {
        vec![format!(
            "{}: {}",
            key,
            rendered_lines.first().copied().unwrap_or("")
        )]
    };

    let mut out: Vec<String> = Vec::new();
    let mut replaced = false;
    let mut iter = yaml.lines().peekable();

    while let Some(line) = iter.next() {
        if !replaced && is_top_level_key(line, key) {
            out.extend(replacement.iter().cloned());
            replaced = true;
            while let Some(next) = iter.peek() {
                if next.starts_with(' ') || next.starts_with('\t') {
                    iter.next();
                } else {
                    break;
                }
            }
        } else {
            out.push(line.to_string());
        }
    }

    if !replaced {
        out.extend(replacement.iter().cloned());
    }

    let mut result = out.join("\n");
    if !result.is_empty() && !result.ends_with('\n') {
        result.push('\n');
    }
    result
}

pub fn set_yaml_block(yaml: &str, key: &str, block: &str) -> String {
    let block = block.trim_end_matches('\n');
    let block_lines: Vec<&str> = block.lines().collect();

    let mut out: Vec<String> = Vec::new();
    let mut replaced = false;
    let mut iter = yaml.lines().peekable();

    while let Some(line) = iter.next() {
        if replaced || !is_top_level_key(line, key) {
            out.push(line.to_string());
            continue;
        }
        out.extend(block_lines.iter().map(|line| (*line).to_string()));
        replaced = true;
        while let Some(next) = iter.peek() {
            if !(next.starts_with(' ') || next.starts_with('\t')) {
                break;
            }
            iter.next();
        }
    }

    if !replaced {
        out.extend(block_lines.iter().map(|line| (*line).to_string()));
    }

    let mut result = out.join("\n");
    if !result.is_empty() && !result.ends_with('\n') {
        result.push('\n');
    }
    result
}
