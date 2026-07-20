---
title: Enforce single .wm/ directory invariant — CI check + lint rule
type: task
status: todo
tags: [spec, architecture, invariant, ci]
relates_to:
  - {type: implements, target: wiki:specs:single-wm-directory-invariant}
---

**Severity:** Medium

**Spec:** @wiki/specs/single-wm-directory-invariant

## Acceptance Criteria

- [ ] AC-1: `wm lint check` detects `.wm/` directories outside project root and warns
- [ ] AC-2: CI pipeline runs `find . -not -path './.wm' -name '.wm' -type d` and fails if any rogue `.wm/` exists
- [ ] AC-3: All code uses `project_root.join(".wm")` — verify by code search
- [ ] AC-4: No crate creates `.wm/` locally — verify by `find` across workspace
- [ ] AC-5: `cargo build` + `cargo test` green

## Files

- `apps/wm-core/src/mcp/tools/lint.rs` — add rogue `.wm/` check
- `justfile` — add CI check step
