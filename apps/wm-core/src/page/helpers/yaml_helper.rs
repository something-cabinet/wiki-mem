use super::frontmatter_value::FrontmatterValue;

const FRONTMATTER_INDENT: &str = "  ";
const ACCEPTANCE_CRITERIA_KEY: &str = "acceptance_criteria";
const CHECKED_KEY: &str = "checked";
const ITEM_MARKER: char = '-';

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

fn is_block_continuation(line: &str) -> bool {
    line.starts_with(' ') || line.starts_with('\t') || line.trim().is_empty()
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
                if !is_block_continuation(next) {
                    break;
                }
                iter.next();
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
    let mut out: Vec<String> = Vec::new();
    let mut scan = AcceptanceCriteriaScan::default();
    for line in yaml.lines() {
        scan.absorb(line, index, checked, &mut out);
    }
    scan.close_target(checked, &mut out);

    let mut result = out.join("\n");
    if !result.is_empty() && !result.ends_with('\n') {
        result.push('\n');
    }
    result
}

#[derive(Default)]
struct AcceptanceCriteriaScan {
    in_block: bool,
    item_count: usize,
    target_open: bool,
    completed: bool,
    target_indent: Option<usize>,
}

impl AcceptanceCriteriaScan {
    fn absorb(&mut self, line: &str, index: usize, checked: bool, out: &mut Vec<String>) {
        if !self.in_block {
            out.push(line.to_string());
            if is_top_level_key(line, ACCEPTANCE_CRITERIA_KEY) {
                self.in_block = true;
            }
            return;
        }

        let trimmed = line.trim_start();
        if is_block_terminator(line, trimmed) {
            self.close_target(checked, out);
            self.in_block = false;
            out.push(line.to_string());
            return;
        }

        if trimmed.starts_with(ITEM_MARKER) {
            self.close_target(checked, out);
            self.open_item(line, index, checked, out);
            return;
        }

        if self.target_open && !self.completed {
            self.absorb_target_line(line, trimmed, checked, out);
            return;
        }

        out.push(line.to_string());
    }

    fn open_item(&mut self, line: &str, index: usize, checked: bool, out: &mut Vec<String>) {
        let is_target = self.item_count == index;
        self.item_count = self.item_count.wrapping_add(1);

        if !is_target {
            out.push(line.to_string());
            return;
        }

        if let Some(rewritten) = rewrite_flow_item(line, checked) {
            out.push(rewritten);
            self.completed = true;
            return;
        }

        self.target_open = true;
        self.target_indent = Some(leading_indent(line));
        out.push(line.to_string());
    }

    fn absorb_target_line(
        &mut self,
        line: &str,
        trimmed: &str,
        checked: bool,
        out: &mut Vec<String>,
    ) {
        if !trimmed.starts_with(CHECKED_KEY) {
            out.push(line.to_string());
            return;
        }

        out.push(render_checked_line(leading_indent(line), checked));
        self.completed = true;
    }

    fn close_target(&mut self, checked: bool, out: &mut Vec<String>) {
        if self.target_open && !self.completed {
            if let Some(indent) = self.target_indent {
                out.push(render_checked_line(
                    indent + FRONTMATTER_INDENT.len(),
                    checked,
                ));
                self.completed = true;
            }
        }

        self.target_open = false;
        self.target_indent = None;
    }
}

fn leading_indent(line: &str) -> usize {
    line.len() - line.trim_start().len()
}

fn is_block_terminator(line: &str, trimmed: &str) -> bool {
    leading_indent(line) == 0 && !trimmed.is_empty() && !trimmed.starts_with(ITEM_MARKER)
}

fn render_checked_line(indent: usize, checked: bool) -> String {
    format!("{}{}: {}", " ".repeat(indent), CHECKED_KEY, checked)
}

fn rewrite_flow_item(line: &str, checked: bool) -> Option<String> {
    let body_start = flow_body_start(line)?;
    let body = &line[body_start..];
    let close = body.rfind('}')?;
    let inner = &body[1..close];
    let rewritten = set_flow_checked(inner, checked);
    Some(format!(
        "{}{{{}}}{}",
        &line[..body_start],
        rewritten,
        &body[close + 1..]
    ))
}

fn flow_body_start(line: &str) -> Option<usize> {
    let marker_at = line.find(ITEM_MARKER)?;
    let after_marker = marker_at + ITEM_MARKER.len_utf8();
    let rest = &line[after_marker..];
    let body_start = after_marker + (rest.len() - rest.trim_start().len());

    match line.as_bytes().get(body_start) {
        Some(b'{') => Some(body_start),
        Some(_) | None => None,
    }
}

fn set_flow_checked(inner: &str, checked: bool) -> String {
    let rendered = checked.to_string();
    if let Some((start, end)) = checked_value_span(inner) {
        return format!("{}{}{}", &inner[..start], rendered, &inner[end..]);
    }

    let trimmed = inner.trim_end();
    if trimmed.is_empty() {
        return format!("{}: {}", CHECKED_KEY, rendered);
    }

    format!("{}, {}: {}", trimmed, CHECKED_KEY, rendered)
}

fn checked_value_span(inner: &str) -> Option<(usize, usize)> {
    let mut search_from = 0usize;
    loop {
        let relative = inner[search_from..].find(CHECKED_KEY)?;
        let key_start = search_from + relative;
        let key_end = key_start + CHECKED_KEY.len();
        let after = &inner[key_end..];

        if is_key_boundary(inner, key_start) && has_key_colon(after) {
            let colon = after.find(':').unwrap_or(0);
            return Some(value_span(inner, key_end + colon + 1));
        }

        search_from = key_end;
    }
}

fn is_key_boundary(inner: &str, key_start: usize) -> bool {
    if is_inside_quotes(inner, key_start) {
        return false;
    }

    match inner.as_bytes().get(key_start.wrapping_sub(1)) {
        None => true,
        Some(b' ') | Some(b',') | Some(b'{') => true,
        Some(_) => false,
    }
}

fn is_inside_quotes(inner: &str, at: usize) -> bool {
    inner[..at].matches('"').count() % 2 == 1
}

fn has_key_colon(after: &str) -> bool {
    match after.find(':') {
        Some(colon) => after[..colon].trim().is_empty(),
        None => false,
    }
}

fn value_span(inner: &str, value_start: usize) -> (usize, usize) {
    let after_colon = &inner[value_start..];
    let end_rel = after_colon.find(',').unwrap_or(after_colon.len());
    let value_end = value_start + end_rel;
    let segment = &inner[value_start..value_end];
    let lead = segment.len() - segment.trim_start().len();
    (value_start + lead, value_end)
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
