---
title: Durable stale marker for a failed derived-state refresh
type: memory
status: active
relates_to:
  - {type: references, target: wiki:patterns:durable-stale-marker-for-failed-refresh}
tags: [error-handling, index, cache, reliability, diagnostics]
---

A failed watcher refresh must not be log-and-drop: classify the error (retry only lock contention, e.g. 'Locking error' / 'locked by another process'), name the lock holder (lsof/ps at the DB-open boundary, best-effort), retry bounded (3 attempts, 100ms x4), persist a stale marker (.wm/state/code-index.stale) on exhaustion, have is_stale honor it, clear it on success (a clear failure is logged and non-fatal), and expose it via wm_index_status (code_index_stale, code_index_refresh_error). Full reference: @wiki/patterns/durable-stale-marker-for-failed-refresh