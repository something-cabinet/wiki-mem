---
title: 'Pattern: Identical-Function → Generic Composition'
type: pattern
id: wiki:patterns:identical-function-composition
relates_to:
  - {type: references, target: wiki:tasks:task-uc9ioi-architectural-refactors-toolsrs-split-skill-dependency-method-extraction}
---
id: wiki:patterns:identical-function-composition

---
id: wiki:patterns:identical-function-composition
title: Pattern: Identical-Function → Generic Composition
type: pattern
tags: [pattern, refactoring, boilerplate, rust]
---
id: wiki:patterns:identical-function-composition

## Problem

Multiple functions with identical structure (same control flow, same error handling, same result building) that only differ by data (language variant, mapper reference, extension string). Each new variant requires copy-pasting the entire block.

## Solution

Extract a private generic function parameterized over the varying data:

```rust
fn for_language(
    source: &str, file: &str, language: &str,
    lang: &SupportedLanguage, ext: &str,
    queries: &[(&str, &'static str)],
) -> Vec<CodeIntelSymbol> {
    let mut results = Vec::new();
    let Ok(tree) = parse_source(source, ext) else { return results };
    for (query_str, kind) in queries {
        if let Ok(cq) = compile_query(lang, query_str, kind) {
            for (name, line, col, _sb, _eb) in run_query(&cq.query, cq.name_index, tree.root_node(), source.as_bytes()) {
                let snippet = get_line_at_offset(source, _sb).trim().to_string();
                results.push(CodeIntelSymbol {
                    name, kind: kind.to_string(), file: file.to_string(),
                    line, column: col, snippet, language: language.to_string(),
                });
            }
        }
    }
    results
}
```

Each language variant becomes a thin wrapper that only defines its queries:

```rust
pub(crate) fn for_rust(source: &str, file: &str, language: &str) -> Vec<CodeIntelSymbol> {
    for_language(source, file, language, &SupportedLanguage::Rust, "rs", &[
        (r"(function_item name: (identifier) @name)", "function"),
        // ...
    ])
}
```

## When to Use

- 3+ functions with identical structure, only data varies
- Each function is 15+ lines of duplicated control flow
- A table/dict of per-variant data can describe all differences

## When Not to Use

- Functions differ in their fundamental logic (not just data)
- Only 2 variants (copy-paste is sometimes clearer)
- The data can't be expressed as a static mapping

## Signals

- Copy-paste with minor edits across 3+ files or functions
- You add a new variant and find yourself re-reading the existing ones to get the structure right
- Review comments say "this is the same as the function above"

## Related