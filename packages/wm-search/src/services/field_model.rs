use std::collections::HashMap;

/// Snowball (Porter2) English stemmer for full-text search token normalization.
static STEMMER: std::sync::LazyLock<rust_stemmers::Stemmer> =
    std::sync::LazyLock::new(|| rust_stemmers::Stemmer::create(rust_stemmers::Algorithm::English));

/// A weighted field within a searchable document
#[derive(Debug, Clone)]
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
///
/// Applies Snowball English stemming to each sub-token, adding the stemmed form
/// alongside the original (when they differ). This enables matching across
/// morphological variants — e.g., "patterns" ↔ "pattern", "designer" ↔ "design",
/// "queries" ↔ "query". Term frequencies are preserved (no dedup) so BM25's TF
/// saturation works correctly.
pub fn tokenize(text: &str) -> Vec<String> {
    let lower = text.to_lowercase();
    let mut tokens = Vec::new();

    // Pass 1: extract full identifiers
    static TOKEN_RE: std::sync::LazyLock<regex::Regex> = std::sync::LazyLock::new(|| {
        regex::Regex::new(r"[a-z0-9_\-]+").expect("hardcoded field name pattern should be valid")
    });
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
                // Pass 3: Snowball stem — push stemmed form only when different from original
                let stemmed = STEMMER.stem(part).to_string();
                if stemmed != part {
                    tokens.push(stemmed);
                }
            }
        }
    }

    tokens
}

/// Stem a single word using the Snowball English stemmer.
/// Used by rerank_boost for exact-match comparison across morphological variants.
/// Returns the stemmed form (e.g., "patterns" → "pattern", "designer" → "design").
pub fn stem_word(word: &str) -> String {
    STEMMER.stem(word).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_stemming_plural() {
        // "patterns" → also produces "pattern"
        let tokens = tokenize("design patterns");
        assert!(tokens.contains(&"design".to_string()));
        assert!(tokens.contains(&"patterns".to_string()));
        assert!(
            tokens.contains(&"pattern".to_string()),
            "stemmed from 'patterns'"
        );
    }

    #[test]
    fn test_tokenize_reference_page() {
        // The headline use case: Design Patterns Reference
        let tokens = tokenize("Design Patterns Reference");
        assert!(tokens.contains(&"design".to_string()));
        assert!(
            tokens.contains(&"pattern".to_string()),
            "stemmed from 'Patterns'"
        );
        assert!(
            tokens.contains(&"patterns".to_string()),
            "original plural kept"
        );
        assert!(tokens.contains(&"reference".to_string()));
    }

    #[test]
    fn test_tokenize_designer() {
        // Snowball stems "designer" → "design"
        let tokens = tokenize("Designer Review");
        assert!(tokens.contains(&"designer".to_string()));
        assert!(
            tokens.contains(&"design".to_string()),
            "stemmed from 'designer'"
        );
    }

    #[test]
    fn test_tokenize_ies_ending() {
        // Snowball handles -ies → -i (Porter algorithm): "queries" → "queri"
        let tokens = tokenize("queries");
        assert!(tokens.contains(&"queries".to_string()));
        assert!(
            tokens.contains(&"queri".to_string()),
            "stemmed from 'queries' via Porter"
        );
    }

    #[test]
    fn test_tokenize_compound_identifiers() {
        // Code-aware: compound identifiers still work alongside stemming
        let tokens = tokenize("ERR_AUTH_401");
        assert!(tokens.contains(&"err_auth_401".to_string()));
        assert!(tokens.contains(&"auth".to_string()));
        assert!(tokens.contains(&"err".to_string()));
    }

    #[test]
    fn test_term_frequency_preserved() {
        // No global dedup: term frequencies must accumulate correctly
        let f = Field::new("body", "design design design", 1.0);
        assert_eq!(
            f.term_freqs.get("design"),
            Some(&3.0),
            "tf preserved across repeated tokens"
        );
    }

    #[test]
    fn test_tokenize_bare_plural() {
        // "patterns" alone produces both forms
        let tokens = tokenize("patterns");
        assert!(tokens.contains(&"patterns".to_string()));
        assert!(tokens.contains(&"pattern".to_string()));
    }

    #[test]
    fn test_tokenize_singular_unchanged() {
        // Already-singular words should not produce duplicate stems
        let tokens = tokenize("design pattern reference");
        let count_design = tokens.iter().filter(|t| *t == "design").count();
        let count_pattern = tokens.iter().filter(|t| *t == "pattern").count();
        let count_reference = tokens.iter().filter(|t| *t == "reference").count();
        assert_eq!(
            count_design, 1,
            "design appears exactly once (stem == original)"
        );
        assert_eq!(
            count_pattern, 1,
            "pattern appears exactly once (stem == original)"
        );
        assert_eq!(
            count_reference, 1,
            "reference stem 'refer' is different, so reference appears once + refer once"
        );
    }

    #[test]
    fn test_short_words_not_stemmed() {
        // Single-char tokens filtered by sub-token filter (part.len() > 1).
        // 2-char words ("is", "at") pass through but don't get stemmed (stem same as original).
        let tokens = tokenize("a is at");
        assert!(!tokens.contains(&"a".to_string()), "single char filtered");
    }

    #[test]
    fn test_tokenize_ing_ending() {
        // Snowball stems -ing: "styling" → "style"
        let tokens = tokenize("styling");
        assert!(tokens.contains(&"styling".to_string()));
        assert!(
            tokens.contains(&"style".to_string()),
            "stemmed from 'styling'"
        );
    }

    #[test]
    fn test_tokenize_ed_ending() {
        // Snowball stems -ed: "rounded" → "round"
        let tokens = tokenize("rounded");
        assert!(tokens.contains(&"rounded".to_string()));
        assert!(
            tokens.contains(&"round".to_string()),
            "stemmed from 'rounded'"
        );
    }

    #[test]
    fn test_tokenize_ly_ending() {
        // Snowball stems -ly
        let tokens = tokenize("softly");
        assert!(tokens.contains(&"softly".to_string()));
        // Snowball stems -ly → -li, but the exact form depends on the word.
        // What matters is that the stemmed form differs from original.
        let stems: Vec<_> = tokens.iter().filter(|t| *t != "softly").collect();
        assert!(
            !stems.is_empty(),
            "should produce at least one stemmed variant of 'softly'"
        );
    }

    #[test]
    fn test_tokenize_compound_with_plural() {
        // Compound identifiers: "design-patterns" → sub-tokens include stemmed "pattern"
        let tokens = tokenize("design-patterns");
        assert!(
            tokens.contains(&"design-patterns".to_string()),
            "full identifier preserved"
        );
        assert!(tokens.contains(&"design".to_string()), "sub-token 'design'");
        assert!(
            tokens.contains(&"patterns".to_string()),
            "sub-token 'patterns'"
        );
        assert!(
            tokens.contains(&"pattern".to_string()),
            "stemmed from 'patterns'"
        );
    }

    #[test]
    fn test_tokenize_mixed_singular_plural() {
        // Searching "pattern" should match "patterns" and "pattern" in docs
        let q_tokens = tokenize("pattern");
        assert!(q_tokens.contains(&"pattern".to_string()));

        let q_tokens = tokenize("patterns");
        assert!(q_tokens.contains(&"patterns".to_string()));
        assert!(
            q_tokens.contains(&"pattern".to_string()),
            "stemmed form in query too"
        );
    }
}
