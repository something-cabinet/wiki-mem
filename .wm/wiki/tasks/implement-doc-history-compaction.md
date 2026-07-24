---
title: "Implement doc history compaction"
type: task
status: todo
priority: low
labels: [refactor, versioning]
---

## Description

The `VersionStoreRepository` has a placeholder comment `// TODO: implement doc history compaction (mirrors compact_task_history)`. The task history compaction logic exists and works; this mirrors that pattern for doc version history.

## Acceptance Criteria

- [ ] Doc history compaction mirrors the task history compaction pattern
- [ ] Old doc versions beyond a configurable limit are compacted
- [ ] No regressions in version history retrieval
