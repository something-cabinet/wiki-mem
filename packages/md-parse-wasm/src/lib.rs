use serde::Serialize;
use wasm_bindgen::prelude::*;

#[derive(Serialize)]
struct ParsedDoc {
    frontmatter: String,
    body: String,
}

/// Extract YAML frontmatter and body from a markdown string.
/// Frontmatter is the block between `---` delimiters at the start of the file.
#[wasm_bindgen]
pub fn parse_markdown(text: &str) -> String {
    let text = text.trim_start();

    if text.starts_with("---") {
        let after_first = &text[3..];
        if let Some(end) = after_first.find("\n---") {
            let yaml = after_first[..end].trim().to_string();
            let body = after_first[end + 4..].trim().to_string();
            let parsed = ParsedDoc {
                frontmatter: yaml,
                body,
            };
            return serde_json::to_string(&parsed).unwrap_or_else(|_| {
                serde_json::to_string(&ParsedDoc {
                    frontmatter: String::new(),
                    body: text.to_string(),
                })
                .unwrap()
            });
        }
    }

    serde_json::to_string(&ParsedDoc {
        frontmatter: String::new(),
        body: text.to_string(),
    })
    .unwrap()
}

/// Parse just the frontmatter and return key-value pairs from YAML.
/// This is a simple line-by-line parser (no full YAML dep needed).
#[wasm_bindgen]
pub fn parse_frontmatter(text: &str) -> String {
    let text = text.trim_start();

    if text.starts_with("---") {
        let after_first = &text[3..];
        if let Some(end) = after_first.find("\n---") {
            let yaml = after_first[..end].trim();
            let mut result = std::collections::HashMap::new();

            for line in yaml.lines() {
                if let Some(pos) = line.find(':') {
                    let key = line[..pos].trim().to_string();
                    let value = line[pos + 1..].trim().to_string();
                    result.insert(key, value);
                }
            }

            return serde_json::to_string(&result).unwrap_or_else(|_| "{}".to_string());
        }
    }

    "{}".to_string()
}
