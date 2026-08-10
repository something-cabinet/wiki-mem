---
title: 'Decision: Never Swallow Parse Errors Silently'
id: wiki:decisions:silent-err-catch-in-parsers
type: decision
relates_to:
  - {type: references, target: wiki:specs:rebuild-log-findings}
status: approved
tags: [decision, parser, error-handling, debugging]
---

## Context

The `extract_frontmatter` function in `apps/wm-core/src/parser/mod.rs` used `Err(_)` to catch serde_yaml deserialization failures. When the `RuleCategory` enum was missing `workflow` and `quality` variants, the entire YAML frontmatter silently failed to parse. This caused 4 rule files to be classified as `concept` instead of `rule` in the graph.

The error was invisible — no log, no warning, no error message. It was discovered only by manually cross-referencing `wm_page.list({"type": "rule"})` (4 results) against the actual files in `.wm/wiki/rules/` (8 files).

## Decision

Never use `Err(_)` in parsers. Every parse failure must be logged with `tracing::warn!` at minimum, including the error message and a snippet of the input.

```rust
// Before — silent, impossible to debug
Err(_) => (None, content),

// After — logged, debuggable
Err(e) => {
    tracing::warn!("Frontmatter parse error: {} — content: {}", e, &content[..content.len().min(100)]);
    (None, content)
}
```

## Rationale

- `Err(_)` makes debugging require manual source inspection or cross-referencing file counts
- Parser errors affect the graph, search, and all downstream features
- A `tracing::warn!` log line costs nothing in production but saves 30+ minutes of debugging
- The `RuleCategory` enum gap was a compile-time bug that manifested as a silent runtime data corruption

## Consequences

- Future parse failures will appear in `wm index rebuild` logs
- Debugging a misclassified page goes from "cross-reference 8 files vs 4 graph entries" to "check the rebuild log"
- All existing `Err(_)` patterns in the codebase should be migrated to `Err(e)` with logging

## Related

- @wiki/rules/no-warnings
- @wiki/patterns/page-type-registration-touch-points
- @wiki/memory/rulecategory-enum-invalid-category-silently-drops-frontmatter