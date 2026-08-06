---
title: WM-002 — Arbitrary file write outside project root via template runner
type: task
status: todo
acceptance_criteria:
  - text: "Path traversal via variables.name (e.g. \"../../x\"), path \"{{name}}\" and destination \"../../..\" is rejected with Err, and addMany traversal is also rejected"
  - text: "All four write actions (add, addMany, modify, append) confine output to the project root"
  - text: "Benign template runs produce byte-identical output to pre-change, rejections name the offending variable and emit tracing::warn!, and cargo clippy --workspace + cargo check --workspace emit zero warnings"
---


Severity: Critical

The template runner substitutes caller-supplied `variables` into destination paths via `render_path` with no sanitisation, then joins without confinement. Verified live: `variables.name = "../../fakehome/.zshrc-pwn"` created a file outside the project root.

Escalates to code execution when a template's path pattern is `{{name}}`, giving full control of directory, basename and extension — targets include shell rc files, git hooks, and `authorized_keys`. A second vector is `config.destination`, which comes from a repo-supplied `_template.yaml`, so a hostile clone is sufficient.

## Acceptance Criteria

- [ ] RED: `variables.name = "../../x"` returns `Err`, failing before the fix
- [ ] `path: "{{name}}"` with a traversing variable is rejected
- [ ] `destination: "../../.."` is rejected
- [ ] `addMany` traversal is rejected
- [ ] All four write actions confine: `add`, `addMany`, `modify`, `append`
- [ ] Benign template runs produce byte-identical output to pre-change
- [ ] The error names the offending variable, not just a resolved path
- [ ] A rejection emits `tracing::warn!`
- [ ] `cargo clippy --workspace` and `cargo check --workspace` emit zero warnings

## Files

- `apps/wm-core/src/mcp/tools/template/mod.rs` (writes at :346 `add`, :385 `addMany`, :452 `modify`, :491 `append`; `destination` at :287-290; `render_path` at :527-552)

## Notes

Depends on the path confinement helper task. `serde_yaml` is deprecated and used by this config loader, so the hygiene task's migration is covered by these tests.
