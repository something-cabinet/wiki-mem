---
title: wm_task stale for new pages — wm_page.update is the authoritative write
type: memory
tags: [tool-reliability, workaround, mcp]
status: active
---

When wm_task.update/get/time misbehave on a freshly created task (phantom NOT_FOUND, stale todo→done transition rejection), use wm_page.update with the same wiki:tasks:... ID as the authoritative write — it resolves and persists. Link the task to its spec to unblock task-store ID resolution. Full writeup: @wiki/concepts/wm-task-store-stale-for-new-pages