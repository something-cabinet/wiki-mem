use std::collections::HashMap;

/// A weighted field within a searchable document
#[derive(Clone, Debug)]
pub struct Field {
    pub name: String,
    pub text: String,
    pub weight: f64,
    pub tokens: Vec<String>,
    pub term_freqs: HashMap<String, f64>,
}

impl Field {
    pub fn new(name: &str, text: &str, weight: f64) -> Self {
        let tokens = tokenize(text);
        let mut term_freqs = HashMap::new();
        for t in &tokens {
            *term_freqs.entry(t.clone()).or_insert(0.0) += 1.0;
        }
        Self {
            name: name.to_string(),
            text: text.to_string(),
            weight,
            tokens,
            term_freqs,
        }
    }
}

/// Code-aware tokenizer: preserves identifiers + sub-tokenizes on _ and -
pub fn tokenize(text: &str) -> Vec<String> {
    let lower = text.to_lowercase();
    let mut tokens = Vec::new();

    // Pass 1: extract full identifiers
    static TOKEN_RE: std::sync::LazyLock<regex::Regex> =
        std::sync::LazyLock::new(|| regex::Regex::new(r"[a-z0-9_\-]+").unwrap());
    for word in TOKEN_RE.find_iter(&lower) {
        let w = word.as_str();

        // Always add the full identifier if it has _ or -
        if w.contains('_') || w.contains('-') {
            tokens.push(w.to_string());
        }

        // Pass 2: sub-tokenize on _ and -
        for part in w.split(&['_', '-'][..]) {
            if !part.is_empty() && part.len() > 1 {
                tokens.push(part.to_string());
            }
        }
    }

    tokens
}
