use serde_json::Value;

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
