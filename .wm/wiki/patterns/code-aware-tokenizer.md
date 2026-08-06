---
title: 'Pattern: Code-Aware Two-Pass Tokenizer'
type: pattern
id: wiki:patterns:code-aware-tokenizer
relates_to:
  - {type: references, target: wiki:reference:search-scoring-formula}
---
id: wiki:patterns:code-aware-tokenizer

---
id: wiki:patterns:code-aware-tokenizer
title: Pattern: Code-Aware Two-Pass Tokenizer
type: pattern
tags: [search, bm25, tokenization, code]
status: reviewed
confidence: high
relates_to:
  - {type: part_of, target: wiki:concepts:bm25-search}
  - {type: example_of, target: wiki:patterns:field-weighted-bm25}
  - {type: references, target: wiki:tasks:g2gckv}
  - {type: references, target: wiki:reference:scoring-config}
  - {type: references, target: wiki:reference:search-scoring-formula}
---
id: wiki:patterns:code-aware-tokenizer

## How it works

Three-pass tokenizer:

**Pass 1 — Extract full identifiers:** Match `[a-z0-9_-]+` patterns from lowercased text. If the token contains `_` or `-`, emit the full identifier as a single token.

**Pass 2 — Sub-tokenize:** Split the full identifier on `_` and `-` boundaries. Emit each component as a separate token.

**Pass 3 — Snowball English stemming (rust-stemmers, Porter2):** For each sub-token, also push its stemmed form when it differs from the original. This normalizes morphological variants so searching "pattern" finds "patterns" and vice versa.

```rust
fn tokenize(text: &str) -> Vec<String> {
    let lower = text.to_lowercase();
    let mut tokens = Vec::new();
    let re = Regex::new(r"[a-z0-9_\-]+").unwrap();
    for word in re.find_iter(&lower) {
        if word.as_str().contains('_') || word.as_str().contains('-') {
            tokens.push(word.as_str().to_string());         // full identifier
        }
        for part in word.as_str().split(&['_', '-'][..]) {
            if !part.is_empty() && part.len() > 1 {
                tokens.push(part.to_string());               // sub-token
                // Pass 3: Snowball stem (added 2026-07-24)
                let stemmed = STEMMER.stem(part).to_string();
                if stemmed != part {
                    tokens.push(stemmed);
                }
            }
        }
    }
    tokens
}
```

**Result:** `ERR_AUTH_401` → `["err_auth_401", "err", "auth", "401"]`  
**Stemming:** `"Design Patterns Reference"` → `["design", "patterns", "pattern", "reference"]`

### Morphological coverage

| Suffix | Example | Stem |
|--------|---------|------|
| Plural -s | patterns → pattern | ✓ |
| -ies | queries → queri | ✓ |
| -er | designer → design | ✓ |
| -ing | styling → style | ✓ |
| -ed | rounded → round | ✓ |
| -ly | softly → softli | ✓ |

### Rerank integration

The exact-match rerank boost (+8.0) uses the same Snowball stemmer on both query and title, so stemmed variants like "patterns"↔"Pattern" still get the full boost. Prefix/starts_with checks (+4.0) use raw strings (character-level) so they work independently of stemming.