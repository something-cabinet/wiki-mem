---
title: "Wiki Tool Reliability: wm_task.update — status transition + frontmatter corruption"
id: wm-task-update-frontmatter-corruption
type: task
status: done
priority: high
tags: [bug, tool-reliability, task-store]
implementation_notes: "Additional evidence 2026-07-31: task wiki:tasks:wm-index-code-output-misleading--report-totals-make---skip-hash-check-force-re-parse went through wm_task.update(status, implementation_plan, append_notes) x3 during its lifecycle. Final file frontmatter contained ONLY status: done + implementation_notes — id:/title:/type: were stripped by the update path (same root cause as this issue). Verify wm_task.update preserves all frontmatter fields. FINAL VALIDATION 2026-08-10: frontmatter preservation + fresh transition reads verified (3 regression tests added to mcp_test.rs, all pass); AC-2 NOT satisfied for graph-indexed tasks — wm_task.update does not refresh the in-memory graph and wm-server runs no file watcher, so wm_task.get returns stale status until wm_index_rebuild (pinned by #[ignore]d test test_regression_wm_task_get_status_fresh_after_index). Task stays in-progress. FIXED 2026-08-10 (same day): all page write paths (update_page_with_repo, create_page_with_repo, delete_page_with_repo) now call graph::handle_file_change/handle_file_delete synchronously after the write, so the in-memory graph snapshot reflects the file immediately — wm-server needs no file watcher. AC-2 regression test un-ignored and passing (mcp_test 74 passed, 0 ignored); daemon-context test daemon_task_update_get_fresh_status_without_rebuild added to e2e_http (11 passed, 1 pre-existing ignore) confirms freshness end-to-end through wm-server. cargo check + clippy -D warnings clean for wm-core and wm-server."
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

## Final Validation (2026-08-10)

Regression tests added to `apps/wm-core/tests/mcp_test.rs` (all passing):
- `test_regression_wm_task_update_roundtrip_preserves_all_fields` — create →
  update(title, status, tags, priority, implementation_plan/notes) → get →
  asserts every field preserved on disk + parses back via
  `extract_frontmatter`, no `{}` block. (AC-3/AC-4 ✓)
- `test_regression_wm_task_update_transition_get_fresh` — todo → in-progress →
  get (immediate) shows in-progress; → done → get (immediate) shows done;
  file stays valid YAML. (AC-1 ✓, AC-2 ✓ for the wm_task.create path)
- `test_regression_wm_task_link_then_update_preserves_frontmatter` — the exact
  bug repro: wm_page.link then wm_task.update → id/title/type/tags/relates_to
  all intact on disk, no `{}`, task still findable via get. (AC-3/AC-4 ✓)
- `test_regression_wm_task_get_status_fresh_after_index` — **un-ignored and
  passing**: get returns the updated status immediately after update for a task
  indexed in the in-memory graph (AC-2 ✓).

Validation results (2026-08-10, after the AC-2 fix):
- `cargo test -p wm-core --test mcp_test` — 74 passed, 0 failed, 0 ignored.
- `cargo test -p wm-core --test e2e_http` — 11 passed, 0 failed, 1 ignored
  (pre-existing `memory_promote` ignore); includes the new daemon-context test
  `daemon_task_update_get_fresh_status_without_rebuild` which confirms
  wm_task.update → wm_task.get returns the new status immediately through
  wm-server (no file watcher, no index rebuild).
- `cargo test -p wm-core --test wm_cli_web_test` — 12 passed, 0 failed.
- `cargo test -p wm-core --lib` — 131 passed, 0 failed.
- `cargo check -p wm-core` / `cargo check -p wm-server` — clean.
- `cargo clippy -p wm-core -- -D warnings` / `cargo clippy -p wm-server -- -D warnings` — clean.

**AC-2 fix (2026-08-10):** the page write paths now refresh the in-memory graph
synchronously after the write, so the daemon needs no file watcher:
- `update_page_with_repo` (page_update_builder_service.rs) — after `repo.write`,
  calls `graph::handle_file_change(&wiki_dir, anchored_path, engine)`.
- `create_page_with_repo` (page_crud_service.rs) — after the write, calls
  `graph::handle_file_change(...)` (previously only the `wm_page.create` tool
  did this; now every create path — wm_task.create, wm_memory, wm_decision —
  refreshes the graph too).
- `delete_page_with_repo` (page_crud_service.rs) — after the remove, calls
  `graph::handle_file_delete(...)`.
- The duplicate `handle_file_change` in `wm_page.create` (mcp/tools/page/mod.rs)
  was removed since `create_page_with_repo` now handles it.
- Added shared helper `page_crud_service::wiki_dir_for(engine)`.

**Task status: `done`** — all ACs verified (AC-2 now fully satisfied for both
the wm_task.create and graph-indexed paths, in both CLI/proxy and daemon
contexts).

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

- [x] AC-1: wm_task.update transition validation reads fresh file state, not the stale graph snapshot (verified: task/mod.rs reads the file fresh; `test_regression_wm_task_update_transition_get_fresh` passes)
- [x] AC-2: wm_task.get returns the updated status immediately after wm_task.update (verified for both the wm_task.create path and graph-indexed tasks, in CLI/proxy AND daemon contexts — write paths call handle_file_change/handle_file_delete; `test_regression_wm_task_get_status_fresh_after_index` un-ignored + passing, daemon e2e `daemon_task_update_get_fresh_status_without_rebuild` passing)
- [x] AC-3: wm_task.update preserves all existing frontmatter fields (id, title, type, tags, priority, relates_to, custom fields) — verified by round-trip + link+update tests
- [x] AC-4: No write path emits a `{}` frontmatter block — verified: no `{}` in any task file, tests assert absence
- [x] AC-5: Corrupted task files from this bug are repaired and index rebuilt — verified: no `{}` blocks or mangled frontmatter in `.wm/wiki/tasks/*.md`
- [x] AC-6: Regression tests cover round-trip, transition, and link+update sequences (red first) — added in mcp_test.rs, all pass
- [x] AC-7: cargo check --workspace + clippy clean; wm_cli_web/other suites still green — `cargo check -p wm-core` + `cargo clippy -p wm-core -- -D warnings` clean; mcp_test 74 pass (0 ignored), lib 131 pass, e2e_http 11 pass + 1 pre-existing ignore, wm_cli_web_test 12 pass, e2e_task/e2e_pages/e2e_graph/e2e_workflow/e2e_mcp/e2e_search/file_watcher_test/cli_test/e2e_memory all green

**Overall: DONE** — AC-2 gap closed by refreshing the graph at the write path (see fix notes above). All acceptance criteria verified; task closed as `done`.

## References

- @wiki/rules/tool-reliability-bug-tracking
- @wiki/rules/check-wm-tool-health-before-work
- apps/wm-core/src/mcp/tools/task/mod.rs — handle_update (line ~432)
- apps/wm-core/src/page/services/page_update_builder_service.rs — update_page_with_repo (line ~33)
- apps/wm-core/src/parser/mod.rs — extract_frontmatter / frontmatter_to_yaml
- apps/wm-core/src/mcp/tools/doc.rs:443,451 — serde_yaml::to_string(&fm) call sites
- @wiki/patterns/page-type-registration-touch-points — struct touch points for flatten
