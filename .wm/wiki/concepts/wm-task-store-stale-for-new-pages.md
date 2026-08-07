---
title: 'Failure: wm_task store stale for newly created pages — use wm_page.update as authoritative write'
id: wiki:concepts:wm-task-store-stale-for-new-pages
type: concept
relates_to:
- type: relates_to
  target: wiki:tasks:7ce26d
status: reviewed
---

---
title: 'Failure: wm_task store stale for newly created pages — resolved by disk fallback'
id: wiki:concepts:wm-task-store-stale-for-new-pages
type: concept
status: reviewed
relates_to:
  - {type: relates_to, target: wiki:tasks:7ce26d}
tags: [failure, tool-reliability, mcp, task-store]
---

# Failure: wm_task store stale for newly created pages — resolved by disk fallback

## What went wrong

During the self-install-removal flow, freshly created wiki pages (a new task, created seconds earlier via `wm_task.create`) were invisible to the task store write paths:

- `wm_task.update` and `wm_time.start` returned `NOT_FOUND` for the brand-new task ID
- `wm_task.get` also returned `NOT_FOUND` for the same ID, while `wm_search.query` and `wm_page.get` resolved it fine
- After linking spec → task (which succeeded), `wm_task.update` started resolving the ID, but the status transition validator then rejected `todo → done` with "Invalid transition: todo → done. Allowed: in-progress, cancelled" — even though an `in-progress` update immediately before it returned success
- `wm_page.update` (the page store) worked as the authoritative write path the whole time

Net effect: ~6 extra tool calls and confusion before identifying the workaround.

## Root cause

The task store and the page store resolve IDs through different paths with different staleness. Write paths (`wm_task.update`, `wm_page.update`, `wm_task.delete`) resolved the page **only through the in-memory graph index** and hard-errored when it was stale — while `get` had a disk fallback. Newly created pages are indexed for search/page reads, but the write handlers read a stale snapshot that doesn't include recently created tasks. This was fixed in @wiki/tasks/7ce26d.

## Resolution (2026-08-07)

The root cause was fixed in task 7ce26d via a shared **graph-index-first, disk-fallback** resolver (`resolve_page_meta` in `page_crud_service.rs`). `wm_page.update`, `wm_task.update/get/delete` now resolve against disk when the index is stale, so pages that exist on disk are never falsely reported "not found". See @wiki/patterns/stale-index-disk-fallback.

The workaround below is **no longer needed** — keep it only as historical context.

## Prevention (historical workaround)

- When `wm_task.*` misbehaves on a task you just created, **use `wm_page.update` with the same `wiki:tasks:...` ID as the authoritative write** — it resolves and persists correctly
- Link the new task to its spec (`wm_page.link`) — this appeared to unblock task-store ID resolution
- Verify with `wm_validate.check({"entity": "wiki:tasks:..."})` — entity validation reads the page store and works

## Time lost

~10 minutes (6+ failed tool calls, retries, and ID-format experiments)

## Related

- @wiki/patterns/stale-index-disk-fallback — the fix as a reusable pattern
- @wiki/tasks/7ce26d — the fix task
- @wiki/tasks/remove-self-install-flow-wm-upgrade-install-module---full-flag