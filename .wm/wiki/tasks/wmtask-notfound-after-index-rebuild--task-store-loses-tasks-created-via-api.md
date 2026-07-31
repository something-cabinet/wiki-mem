---
title: wm_task NOT_FOUND after index rebuild — task store loses tasks created via API
type: task
id: wiki:tasks:wmtask-notfound-after-index-rebuild--task-store-loses-tasks-created-via-api
status: todo
priority: high
tags: [bug, tool-reliability, task-store]
---

A task created via wm_task.create (id: wiki:tasks:bundle-angular-frontend-with-wm-server-for-npm-distribution) was fully usable (get/update/check_ac worked, plan saved). After running wm_index.rebuild (skip_embed=true), wm_task.get and wm_task.update return NOT_FOUND for the same ID even though the file exists at .wm/wiki/tasks/bundle-angular-frontend-with-wm-server-for-npm-distribution.md with correct frontmatter. wm_search finds the page (as a page), but the task store cannot resolve it. check_ac returned success earlier but the on-disk acceptance_criteria still show checked: false — AC state did not persist to the file.

Impact: cannot transition task to done, cannot append implementation notes, AC check state lost. Task store appears to cache task metadata at startup; index rebuild drops tasks not present at cache-build time, or the task store's id_index is stale.

Repro:
1. wm_task.create → OK
2. wm_task.update (status in-progress, assignee) → OK
3. wm_task.check_ac ×4 → OK (returns checked arrays)
4. wm_index.rebuild (skip_embed=true)
5. wm_task.get → NOT_FOUND (file still on disk)

Expected: task store reads from disk or rebuilds its index after wm_index.rebuild so tasks created via API remain resolvable.