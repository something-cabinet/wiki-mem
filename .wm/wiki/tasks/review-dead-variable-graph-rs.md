---
title: "Cleanup: remove unused _index variable in graph.rs"
type: task
status: done
spec: specs/graph-bugs-review-fixes
tags: [review, backend, cleanup]
priority: low
---

# Cleanup: remove unused `_index` variable in `graph.rs`

## Description

In `apps/wm-core/src/mcp/tools/graph.rs` line 155, `let _index = &snapshot.1;` — the underscore prefix signals "intentionally unused", but the line is still dead weight. If `_index` is shadowed to keep the snapshot borrow alive, add a comment. Otherwise remove it entirely.

## Location

`apps/wm-core/src/mcp/tools/graph.rs` — `graph_full` handler

## Acceptance Criteria

- [ ] Determine if `_index` is needed to keep the `snapshot` borrow alive
- [ ] If needed, add a comment explaining why
- [ ] If not needed, remove the line
