use serde_json::Value;

/// Parse a `@template/name key=val key2=val2` ref into (name, args map).
pub fn parse_template_ref(input: &str) -> (String, serde_json::Map<String, Value>) {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_template_ref() {
        let (name, args) = parse_template_ref("my-template key1=val1 key2=val2");
        assert_eq!(name, "my-template");
        assert_eq!(args.get("key1").and_then(|v| v.as_str()), Some("val1"));
        assert_eq!(args.get("key2").and_then(|v| v.as_str()), Some("val2"));
    }

    #[test]
    fn test_no_args() {
        let (name, args) = parse_template_ref("simple");
        assert_eq!(name, "simple");
        assert!(args.is_empty());
    }
}
