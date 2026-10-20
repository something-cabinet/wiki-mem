---
title: Durable stale marker for a failed derived-state refresh
type: memory
tags: [error-handling, index, cache, reliability]
status: active
relates_to:
  - {type: references, target: wiki:patterns:durable-stale-marker-for-failed-refresh}
---

A failed watcher refresh must not be log-and-drop: retry bounded (3 attempts, 100ms x4), persist a stale marker (.wm/state/code-index.stale) on exhaustion, have is_stale honor it, clear it on success, and expose it via wm_index_status (code_index_stale, code_index_refresh_error). Full reference: @wiki/patterns/durable-stale-marker-for-failed-refresh