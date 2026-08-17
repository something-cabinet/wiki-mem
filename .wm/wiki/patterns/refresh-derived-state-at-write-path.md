---
title: 'Pattern: Refresh derived in-memory state at the write path'
type: pattern
id: wiki:patterns:refresh-derived-state-at-write-path
status: reviewed
tags:
- pattern
- architecture
- graph
- cache
relates_to:
  - {type: references, target: wiki:tasks:wm-task-update-frontmatter-corruption}
---

---
title: 'Pattern: Refresh derived in-memory state at the write path'
type: pattern
id: wiki:patterns:refresh-derived-state-at-write-path
status: reviewed
tags:
- pattern
- architecture
- graph
- cache
relates_to:
  - {type: references, target: wiki:tasks:wm-task-update-frontmatter-corruption}
  - {type: references, target: wiki:specs:code-edge-resolution}
---

## Problem

`wm_task.get` returned stale data right after `wm_task.update` (status stayed `todo` after updating to `in-progress` until an explicit `wm_index_rebuild`). Root cause: the in-memory graph snapshot was only refreshed by the file watcher — which wm-server never runs (it boots `EngineState::new` directly, not `MainEngineFactory`). So `notify_file_changed` fired but had no consumer, and every graph read (`get`/`list`/`board`) hit the stale snapshot.

## Solution

**Refresh the graph snapshot at the write path**, not the read path, and don't depend on the file watcher:

- After every page write (create/update/delete), call `graph::handle_file_change` / `graph::handle_file_delete` synchronously — exactly what the file watcher would have done, but immediate (a watcher would still race its debounce window).
- `create_page_with_repo`, `update_page_with_repo`, `delete_page_with_repo` all do this now (see `apps/wm-core/src/page/services/page_crud_service.rs` + `page_update_builder_service.rs`).
- Result: `wm_task.get`/`list`/`board` and `wm_page.get`/`list` return fresh state in both the CLI/proxy and daemon contexts, with no rebuild required.

## When to Use

Any daemon/service that owns an in-memory graph/index derived from files, where writes go through code paths (not only an external watcher). If a write path mutates state that reads consult, refresh the derived snapshot synchronously in the writer — never assume a watcher/rebuild will catch up.

## When Not to Use

- Read-mostly systems with an active, reliable watcher (the watcher is still fine; the point is not to be the *only* refresh mechanism).
- Write paths that already rebuild the derived index inline.
- **When the tool is NOT the writer** — see D6 Amendment below.

## D6 Amendment (2026-08-17): Code Files

The write-path pattern holds in spirit — derived state refreshes at the moment of change — but the mechanism differs when wm is not the writer:

- **Wiki pages:** wm IS the writer → write-path hook is the correct refresh mechanism.
- **Source code:** wm NEVER writes source code (editors and agent file tools do) → a write-path hook for code files would have no caller and would itself be a dead layer. The watcher is the primary refresh mechanism for code index entries (spec `code-edge-resolution` D6, FR-1.2).

The principle generalizes: **the refresh mechanism must match who writes.** If wm writes it, use a write-path hook. If external tools write it, use a filesystem watcher with a staleness probe for one-shot invocations (`refresh_if_stale`).

## Related

- @task-wm-task-update-frontmatter-corruption (AC-2 fix)
- @wiki/concepts/frontmatter-corruption-sci-notation-id
- @wiki/specs/code-edge-resolution (D6 amendment)
