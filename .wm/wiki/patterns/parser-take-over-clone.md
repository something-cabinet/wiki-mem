---
id: wiki:patterns:parser-take-over-clone
title: "Parser Field Extraction: take() Over clone()"
type: pattern
status: active
category: performance
rationale: "Parser extraction code frequently clones every field from frontmatter when the source is consumed immediately after. Using `take()` or `mem::take()` avoids unnecessary allocations."
see: "@wiki/rules/no-dead-code-clone-scanning"
---
id: wiki:patterns:parser-take-over-clone

## Problem

Parser extraction code in `parser/mod.rs` (and similar patterns) clones each field from a parsed frontmatter struct:

```rust
// Pattern: clone from reference
let fm = parse_frontmatter(content);
let fm_ref = fm.as_ref();  // &Frontmatter

// Each field cloned individually — 15+ clones
let tags = fm_ref.map(|f| f.tags.clone()).unwrap_or_default();
let assignee = fm_ref.and_then(|f| f.assignee.clone());
let version = fm_ref.and_then(|f| f.version.clone());
// ... more clones ...

// fm is then dropped — all those clone allocations were wasted
```

The frontmatter is never used after extraction. Every `.clone()` allocates a new String/ Vec that could have been moved.

## Solution

Take ownership of the frontmatter and use `Option::take()` or `std::mem::take()` to move fields out:

```rust
// Before: 15+ clone calls
fn build_meta(content: &str) -> WikiPageMeta {
    let fm = parse_frontmatter(content);
    WikiPageMeta {
        tags: fm.as_ref().map(|f| f.tags.clone()).unwrap_or_default(),
        assignee: fm.as_ref().and_then(|f| f.assignee.clone()),
        // ...
    }
}

// After: zero clones
fn build_meta(content: &str) -> WikiPageMeta {
    let mut fm = parse_frontmatter(content);
    WikiPageMeta {
        tags: fm.as_mut().map(|f| std::mem::take(&mut f.tags)).unwrap_or_default(),
        assignee: fm.as_mut().and_then(|f| f.assignee.take()),
        // ...
    }
}
```

Or even better, destructure the frontmatter:

```rust
fn build_meta(content: &str) -> WikiPageMeta {
    let Some(fm) = parse_frontmatter(content) else {
        return WikiPageMeta::default();
    };
    // fm consumed — move fields directly
    WikiPageMeta {
        tags: fm.tags,
        assignee: fm.assignee,
        // ...
    }
}
```

## When to Use

- Parsing/extraction where the source is consumed after reading
- Frontmatter parsing, config parsing, deserialization that feeds into another structure
- Any pattern of `let x = source.field.clone()` followed by `drop(source)`

## When Not to Use

- When the source is shared or used again after extraction
- When the field needs to remain in the source for later use (`take()` empties it)
- With `Copy` types (cloning is the same as copying)

## Enforcement

- `rg '\.as_ref\(\)[^)]*\.clone\(\)'` to find clone-from-reference patterns
- Review parser extraction code for `clone()` calls on fields that could be moved
