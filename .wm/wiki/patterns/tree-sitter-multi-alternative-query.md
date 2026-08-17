---
title: 'Pattern: Tree-Sitter Multi-Alternative Query with Shared Captures'
type: pattern
id: wiki:patterns:tree-sitter-multi-alternative-query
status: draft
tags: [pattern, tree-sitter, code-intel, extraction]
relates_to:
  - {type: references, target: wiki:tasks:code-edge-resolution-04-capture-every-call-form-and-the-receiver-expression}
---

## Problem

Tree-sitter queries for a single concept (e.g. "all function calls") need to match multiple AST node shapes: bare calls use `(call_expression function: (identifier))`, method calls use `(call_expression function: (field_expression ...))`, and path calls use `(call_expression function: (scoped_identifier ...))`. Writing separate queries and passes is wasteful and makes the capture-index logic brittle.

## Solution

Combine all alternatives into a single query using tree-sitter's alternation syntax `[...]`, with shared capture names across alternatives:

```rust
let query = r#"[
    (call_expression function: (identifier) @name)
    (call_expression function: (field_expression value: (_) @recv field: (field_identifier) @name))
    (call_expression function: (scoped_identifier path: (_) @recv name: (identifier) @name))
]"#;
```

Use `capture_index_for_name` to look up shared captures (`@name`, `@recv`) — they have the same index across all alternatives. Then iterate matches, extracting whichever captures are present:

```rust
let name_index = query.capture_index_for_name("name");
let recv_index = query.capture_index_for_name("recv");
for match_ in cursor.matches(&query, root, source.as_bytes()) {
    let mut callee_node = None;
    let mut recv_node = None;
    for capture in match_.captures {
        if capture.index == name_index { callee_node = Some(capture.node); }
        if Some(capture.index) == recv_index { recv_node = Some(capture.node); }
    }
    // Process callee_node (always present) and recv_node (optional)
}
```

Optional captures (`@recv` absent in bare calls) simply produce no capture in that match — no special error handling needed.

## When to Use

- Extracting edges, symbols, or metadata that span multiple AST node shapes
- Any tree-sitter extraction where one semantic concept has multiple syntactic forms
- When you need a shared capture name across alternatives for uniform downstream processing

## When Not to Use

- When alternatives have incompatible capture names (different fields needed per shape)
- When performance requires early pruning that tree-sitter alternation doesn't provide

## Language-Specific Notes

| Language | Method calls | Path/namespace calls |
|----------|-------------|---------------------|
| Rust | `field_expression` (value + field_identifier) | `scoped_identifier` (path + name) |
| TypeScript/TSX | `member_expression` (object + property_identifier) | Same as method (member_expression) |
| Python | `attribute` (object + identifier) | N/A (uses dot like methods) |
| Go | `selector_expression` (operand + field_identifier) | Same as method (selector_expression) |

## Related

- @wiki/tasks/code-edge-resolution-04-capture-every-call-form-and-the-receiver-expression — first use
- @wiki/specs/code-edge-resolution — governing spec