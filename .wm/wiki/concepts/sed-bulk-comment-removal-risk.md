---
id: wiki:concepts:sed-bulk-comment-removal-risk
title: "Failure: sed-based Bulk Comment Removal"
type: concept
tags: [failure, refactoring, sed, tooling]
status: draft
relates_to:
  - {type: references, target: wiki:tasks:strip-all-comments-from-source-code}
---
id: wiki:concepts:sed-bulk-comment-removal-risk

## What went wrong

Using `sed -i '' 's!//.*!!'` to bulk-remove `//` comments from Rust source files destroyed every line containing `//` — including string literals like URLs, regular expressions, and base64 data. This broke the build silently.

## Root cause

The sed regex `s!//.*!!` matches `//` anywhere on a line, including inside string literals. Rust's `//` comment syntax is only a comment at the start of a line (after whitespace), not inside quotes. A general-purpose regex cannot distinguish code `//` from comment `//`.

## Prevention

For comment removal in source code:
- Use AST-aware tools (ast-grep) that understand the language grammar
- Or use targeted per-file edits with the edit tool (exact string replacement)
- If using regex, scope to `^\s*//` (line-start only, not inline) to avoid string literals
- Always verify with `cargo check` or `tsc --noEmit` after any bulk edit

## Time lost
~5 minutes to detect and fix (cargo check caught it immediately)

## Related
- @wiki/tasks/c19d50
