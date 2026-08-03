---
{}
relates_to:
  - {type: relates_to, target: wiki:tasks:7ce26d}
---

---
{}
relates_to:
  - {type: references, target: wiki:tasks:remove-self-install-flow-wm-upgrade-install-module---full-flag}
---

---
title: Failure: wm_task store stale for newly created pages — use wm_page.update as authoritative write
type: concept
id: wiki:concepts:wm-task-store-stale-for-new-pages
tags: [failure, tool-reliability, mcp, task-store]
---

## What went wrong

During the self-install-removal flow, freshly created wiki pages (a new task, created seconds earlier via `wm_task.create`) were invisible to the task store write paths:

- `wm_task.update` and `wm_time.start` returned `NOT_FOUND` for the brand-new task ID
- `wm_task.get` also returned `NOT_FOUND` for the same ID, while `wm_search.query` and `wm_page.get` resolved it fine
- After linking spec → task (which succeeded), `wm_task.update` started resolving the ID, but the status transition validator then rejected `todo → done` with "Invalid transition: todo → done. Allowed: in-progress, cancelled" — even though an `in-progress` update immediately before it returned success
- `wm_page.update` (the page store) worked as the authoritative write path the whole time

Net effect: ~6 extra tool calls and confusion before identifying the workaround.

## Root cause

The task store and the page store resolve IDs through different paths with different staleness. Newly created pages are indexed for search/page reads, but the task store's ID lookup and its in-memory status transition validator read a stale snapshot that doesn't include recently created tasks. The transition validator appears to re-read a cached task state, so a "successful" status update doesn't reliably change what the next transition check sees.

## Prevention

- When `wm_task.*` misbehaves on a task you just created, **use `wm_page.update` with the same `wiki:tasks:...` ID as the authoritative write** — it resolves and persists correctly
- Link the new task to its spec (`wm_page.link`) — this appeared to unblock task-store ID resolution
- Verify with `wm_validate.check({"entity": "wiki:tasks:..."})` — entity validation reads the page store and works
- Tracked as part of the known tool-reliability bug set: @wiki/tasks/7ce26d (phantom "page not found" on update, page_id vs id confusion, match-arm value discarding in mcp/tools/page.rs)

## Time lost

~10 minutes (6+ failed tool calls, retries, and ID-format experiments)

## Related

- @wiki/tasks/remove-self-install-flow-wm-upgrade-install-module---full-flag
- @wiki/tasks/7ce26d