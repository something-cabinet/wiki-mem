---
title: Refresh derived in-memory state at the write path
type: memory
tags: [architecture, graph, cache, stale]
status: active
---

If a write path mutates state that reads consult (graph/index), refresh the derived snapshot synchronously in the writer — never rely on a file watcher to catch up (wm-server runs no watcher; wm_task.get stayed stale until rebuild). Full: @wiki/patterns/refresh-derived-state-at-write-path