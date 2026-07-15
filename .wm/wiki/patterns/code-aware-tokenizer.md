---
title: "Pattern: Code-Aware Two-Pass Tokenizer"
type: pattern
tags: [search, bm25, tokenization, code]
status: reviewed
confidence: high
relates_to:
  - {type: part_of, target: "wiki:concepts:bm25-search"}
  - {type: example_of, target: "wiki:patterns:field-weighted-bm25"}
---

## When to use

Any search system indexing technical/code content where identifiers contain underscores (`ERR_AUTH_401`), hyphens (`auth-service`), or camelCase. Standard whitespace + lowercase tokenizers break compound tokens into components, losing the exact match.

## How it works

Two-pass tokenizer:

**Pass 1 — Extract full identifiers:** Match `[a-z0-9_-]+` patterns from lowercased text. If the token contains `_` or `-`, emit the full identifier as a single token.

**Pass 2 — Sub-tokenize:** Split the full identifier on `_` and `-` boundaries. Emit each component as a separate token.

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
            }
        }
    }
    tokens
}
```

**Result:** `ERR_AUTH_401` → `["err_auth_401", "err", "auth", "401"]`

Both exact match and component match work. The rerank booster gives extra weight (+8.0) to exact matches.

## Example
```
Query: "ERR_AUTH_401" → matches "err_auth_401" exactly (high score + boost)
Query: "auth expired"  → matches via "auth" component (lower score, no boost)
```

## Source
@wiki/tasks/g2gckv
