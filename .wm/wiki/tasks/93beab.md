---
title: "Audit: replace sweeping #[allow(dead_code)] with targeted suppression"
id: 93beab
type: task
status: done
tags: [review, backend, cleanup, lint]
priority: high
acceptance_criteria:
  - text: "All #[allow(dead_code)] annotations across the 6 listed files are narrowed to field-level suppression, removed, or the dead code is deleted"
  - text: "Module-level allows are removed from tagged enums (WmIndexAction, WmMemoryAction, WmTimeAction) after confirming serde uses all variants"
  - text: "Uncalled functions like memory_dir()/resolve_root() are either called at a real call site or removed"
---

# Audit: replace sweeping `#[allow(dead_code)]` with targeted suppression

## Description

Nine structs/enums/functions across `apps/wm-core/src/mcp/tools/` and `apps/wm-web/src-tauri/src/commands.rs` have `#[allow(dead_code)]` annotations:

- `WmGraphNeighborsInput`
- `WmIndexAction` enum
- `WmLogRecentInput`, `WmLogSinceInput`, `WmLogFilterInput`
- `WmMemoryAction` enum
- `WmTimeAction` enum
- `memory_dir()` function
- `resolve_root()` function
- `LayoutNode` struct

Most are serde deserialization targets — the lint fires because fields are never read by name in source code. For tagged enums (`WmIndexAction`, `WmMemoryAction`, `WmTimeAction`), the variants ARE used via serde, so suppressing at the enum level is suspicious. For `memory_dir()` and `resolve_root()`, either they're called somewhere or should be removed.

## Location

- `apps/wm-core/src/mcp/tools/graph.rs`
- `apps/wm-core/src/mcp/tools/index.rs`
- `apps/wm-core/src/mcp/tools/log.rs`
- `apps/wm-core/src/mcp/tools/memory.rs`
- `apps/wm-core/src/mcp/tools/time.rs`
- `apps/wm-web/src-tauri/src/commands.rs`

## Acceptance Criteria

- [ ] Audit each `#[allow(dead_code)]` — is it a serde false positive or real dead code?
- [ ] For struct fields: use field-level `#[allow(dead_code)]` instead of module-level
- [ ] For tagged enums: verify serde uses all variants, remove the allow
- [ ] For `memory_dir()` / `resolve_root()`: either call them or remove them
- [ ] For `LayoutNode`: verify it's used in the compute_layout flow
- [ ] Remove any truly dead code instead of suppressing
