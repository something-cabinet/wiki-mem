---
title: 'Pattern: Alias-shim consolidation for duplicate writers'
type: pattern
id: wiki:patterns:alias-shim-consolidation
status: reviewed
tags:
- pattern
- consolidation
- mcp
- refactor
---

## Problem

Two tools write the same underlying format (`.md` + YAML frontmatter) with separate implementations. `wm_doc` carried its own `parse_frontmatter` (duplicating `crate::parser` used elsewhere in the same file) and a `build_markdown` writer that existed solely to byte-imitate `wm_page`'s string-built output — faithfully reproducing quirks and re-introducing the unquoted-tags bug. Parity tests pinned byte-identity of two writers, making consolidation harder (any change to one writer broke the other's tests).

## Solution

Consolidate onto one implementation; keep the redundant tool as a thin alias shim:

1. Extract the shared dispatch: `page::handle_action(&engine, WmPageAction)` — the match body of `register()` becomes a public function.
2. Rewrite the redundant tool's `register()` to map its (backward-compatible) action schema onto the shared action enum via `to_page_action()`, then call `handle_action`.
3. Delete the duplicate writer internals: private `parse_frontmatter`, byte-imitation `build_markdown`, output structs, direct `tokio::fs` I/O.
4. Delete the parity tests — parity now holds by construction (one writer).
5. Preserve behavioral guards the shared path lacks (e.g. `confine_doc_path()` for path-bearing actions where the shared path only confines on create) — port them into the shim.

Result: one writer, ~350 lines removed, issue-126 fixes (type/tags persistence) survive by construction.

## When to Use

- Two tools write the same format with duplicated logic and byte-imitation quirks
- The redundant tool's input schema must stay backward-compatible (skills/web call it)
- Parity tests pin two-writer identity and block refactoring

## When Not to Use

- The redundant tool has genuinely different semantics (then it's not redundant)
- The alias shim would be more code than the writer it replaces (delete outright instead)

## Related

- @wiki/rules/no-compensating-layers — a byte-imitation writer is a compensating layer
- @wiki/specs/retire-wm-doc — the spec this pattern executed
- @task-execute-retire-wm-doc-consolidation