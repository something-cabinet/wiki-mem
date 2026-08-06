---
title: "Wiki Tool Reliability: wm_task.update — status transition + frontmatter corruption"
id: wm-task-update-frontmatter-corruption
type: task
status: in-progress
priority: high
tags: [bug, tool-reliability, task-store]
implementation_notes: "Additional evidence 2026-07-31: task wiki:tasks:wm-index-code-output-misleading--report-totals-make---skip-hash-check-force-re-parse went through wm_task.update(status, implementation_plan, append_notes) x3 during its lifecycle. Final file frontmatter contained ONLY status: done + implementation_notes — id:/title:/type: were stripped by the update path (same root cause as this issue). Verify wm_task.update preserves all frontmatter fields."
acceptance_criteria:
  - text: "wm_task.update transition validation reads fresh file state (not the stale graph snapshot), and wm_task.get returns the updated status immediately after update"
  - text: "wm_task.update preserves all existing frontmatter fields (id, title, type, tags, priority, relates_to, custom fields), and no write path emits a '{}' frontmatter block"
  - text: "Corrupted task files from this bug are repaired with the index rebuilt, and regression tests cover round-trip, transition, and link+update sequences with cargo check + clippy clean"
---

## Bug Description

`wm_task.update` behaves unreliably on task status transitions AND corrupts task frontmatter:

1. **Invalid transition errors**: `wm_task.update({action:"update", id, status:"done"})` on a `todo` task returns `INTERNAL_ERROR: Invalid transition: todo → done. Allowed: in-progress, cancelled`. Updating to `in-progress` first reports success (`{"id":...,"status":"updated"}`) but a subsequent `get` still shows `todo` — the transition validation reads stale state, and even after a successful in-progress update the file is left inconsistent.
2. **Frontmatter corruption**: after `wm_page.link` (adding `relates_to` edges) plus `wm_task.update` calls, the 8 affected task files had their frontmatter mangled — `id`, `title`, `type`, `tags`, `priority` dropped and replaced with a stray `{}` block:
   ```markdown
   ---
   {}
   relates_to:
     - {type: implements, target: wiki:specs:wiki-tool-reliability}
   ---
   ```
   This made `wm_task.get` return `NOT_FOUND` for pages that exist on disk and `wm_task.list` (by label) return empty results.

## Root Cause Analysis (2026-07-31)

### RC-1: Stale graph snapshot for status reads/transitions
`apps/wm-core/src/mcp/tools/task/mod.rs::handle_update` reads `meta.status` from `engine.graph.load()` — the graph snapshot is only rebuilt by `wm_index.rebuild`. After a successful update, `engine.notify_file_changed` only sets a dirty flag; the in-memory graph still holds the OLD status. Consequence:
- transition validation `meta.status.can_transition_to(&parsed)` checks stale state → spurious "Invalid transition" errors
- `wm_task.get` returns stale status → the "get still shows todo" symptom

### RC-2: Frontmatter round-trip drops fields
`apps/wm-core/src/page/services/page_update_builder_service.rs::update_page_with_repo` rebuilds the frontmatter from `crate::parser::frontmatter_to_yaml(existing_fm)`. `frontmatter_to_yaml` emits ONLY a whitelist of modeled fields (title, type, tags, status, priority, ...). serde_yaml ignores unknown keys on parse (default), so fields like `id`, `createdAt`, `updatedAt`, custom fields parse into `None` and are silently dropped on re-serialization. The evidence note confirms: after 3 updates, `id:/title:/type:` were stripped.

### RC-3: `{}` frontmatter emission
Some write path serializes an empty/default `Frontmatter` via `serde_yaml::to_string` → `{}` (empty YAML map), then appends raw fields. Candidates: `apps/wm-core/src/mcp/tools/doc.rs:443,451` (`serde_yaml::to_string(&fm)`), and the `wm_page.link` path. Once a file contains `{}`, subsequent `frontmatter_to_yaml` round-trips only what parses (relates_to), permanently losing the rest.

## Implementation Plan

1. **Fix RC-1 — fresh-state status reads (task/mod.rs)**
   - In `handle_update`, read the file's current frontmatter (fresh, like `update_page_with_repo` already does via `extract_frontmatter`) instead of relying on `meta.status` from the graph for transition validation.
   - Replace the `meta.status.can_transition_to(&parsed)` check with a fresh-file-based check (or move validation entirely into `update_page_with_repo`, which already re-reads the file).
   - Consider refreshing the graph node status after write (or ensure `get`/`list` read fresh), so post-update `get` returns the new status.

2. **Fix RC-2 — lossless frontmatter round-trip (parser + update builder)**
   - Make `frontmatter_to_yaml` preserve unknown fields: either parse frontmatter into a `serde_yaml::Value`/BTreeMap and pass through unknown keys, or add `#[serde(flatten)] unknown: BTreeMap<String, Value>` to `Frontmatter` (flatten pattern — see `@wiki/patterns/page-type-registration-touch-points` for struct touch points) and re-emit them in `frontmatter_to_yaml`.
   - Ensure `id` (and any other task frontmatter field) survives `wm_task.update` round-trips.

3. **Fix RC-3 — never write `{}` frontmatter**
   - Audit `serde_yaml::to_string(&fm)` call sites (doc.rs:443,451 + link path); when the struct is empty/default, either skip writing the frontmatter block or emit the original parsed YAML verbatim.
   - `update_page_with_repo`: if `extract_frontmatter` returns `None` (parse error), bail with a clear error instead of writing a fresh `{}` block that destroys the file.

4. **Repair already-corrupted task files** (8 from the spec run + wm-index-code-output + any others detected):
   - Restore correct frontmatter (title, id, type, status, priority, tags, relates_to) for files whose frontmatter was stripped.
   - Run `wm_index.rebuild` + `wm_index.embed` to resync graph.

5. **Tests (TDD, red first)** — in `apps/wm-core/tests/`:
   - Round-trip: `wm_task.update(status)` on a task with full frontmatter → file still contains id/title/type/tags/priority/relates_to (regression for RC-2/RC-3).
   - Transition: `todo → in-progress` then `in-progress → done` succeed; `get` immediately after shows the new status (regression for RC-1).
   - No `{}`: after update + link, file frontmatter has no `{}` and `wm_task.get` still finds the task.
   - Link+update sequence: `wm_page.link` then `wm_task.update` → task remains findable (the exact repro from the bug report).

6. **Validation**
   - `cargo check --workspace` zero warnings, `cargo clippy --workspace` clean, `cargo test -p wm-core` green.
   - `wm_validate.check({"entity": "tasks/wm-task-update-frontmatter-corruption"})` passes.
   - Manual MCP smoke: create task → update status twice → get → link → update → get.

## Acceptance Criteria

- [ ] AC-1: wm_task.update transition validation reads fresh file state, not the stale graph snapshot
- [ ] AC-2: wm_task.get returns the updated status immediately after wm_task.update
- [ ] AC-3: wm_task.update preserves all existing frontmatter fields (id, title, type, tags, priority, relates_to, custom fields)
- [ ] AC-4: No write path emits a `{}` frontmatter block
- [ ] AC-5: Corrupted task files from this bug are repaired and index rebuilt
- [ ] AC-6: Regression tests cover round-trip, transition, and link+update sequences (red first)
- [ ] AC-7: cargo check --workspace + clippy clean; wm_cli_web/other suites still green

## References

- @wiki/rules/tool-reliability-bug-tracking
- @wiki/rules/check-wm-tool-health-before-work
- apps/wm-core/src/mcp/tools/task/mod.rs — handle_update (line ~432)
- apps/wm-core/src/page/services/page_update_builder_service.rs — update_page_with_repo (line ~33)
- apps/wm-core/src/parser/mod.rs — extract_frontmatter / frontmatter_to_yaml
- apps/wm-core/src/mcp/tools/doc.rs:443,451 — serde_yaml::to_string(&fm) call sites
- @wiki/patterns/page-type-registration-touch-points — struct touch points for flatten
