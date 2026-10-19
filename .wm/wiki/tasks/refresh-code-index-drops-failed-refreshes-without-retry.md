---
title: refresh_code_index_for drops a failed refresh with no retry, losing the index event
type: task
id: "wiki:tasks:refresh-code-index-drops-failed-refreshes-without-retry"
status: todo
tags: [bugfix, code-intel, reliability]
priority: high
acceptance_criteria:
  - {text: "A failed refresh is retried with bounded backoff, or the index is marked stale so the next reader rebuilds", checked: false}
  - {text: "A dropped index event is observable to a caller, not only a log line", checked: false}
  - {text: "A test forces a refresh failure and asserts the event is not silently lost", checked: false}
---

**Finding, surfaced while fixing @wiki/tasks:code-index-watcher-tests-have-a-load-dependent-deadline.**

`refresh_code_index_for` logs a failed refresh and returns, with no retry. A single transient collision with
another writer therefore loses the index event permanently rather than being retried. This is what turned a
DB-lock collision inside a test into a 20-second timeout: the watcher did fire, the refresh failed, and the
event was dropped with only a log line.

**Why it matters.** The code index silently under-reports after a lost event until something else triggers a
full rebuild. Code-intelligence results then disagree with the tree, and nothing surfaces the divergence.

**Fix direction.** Retry a failed refresh with bounded backoff, or mark the index stale so the next reader
rebuilds, rather than discarding the event. Either way the failure must become visible to a caller instead of
a log line.