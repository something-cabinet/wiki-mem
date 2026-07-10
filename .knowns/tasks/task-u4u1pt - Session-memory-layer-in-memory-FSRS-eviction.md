---
id: u4u1pt
title: Session memory layer (in-memory, FSRS eviction)
status: done
priority: high
labels:
  - sprint-0
  - feature
  - memory
createdAt: '2026-07-10T10:14:41.136Z'
updatedAt: '2026-07-10T10:24:00.144Z'
timeSpent: 260
assignee: '@me'
spec: specs/wm-leapfrog-replace-knowns-with-complete-memory-layer
fulfills:
  - AC-2
---
# Session memory layer (in-memory, FSRS eviction)

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Add an in-memory DashMap<String, MemoryEntry> scoped to the MCP server process lifetime. Support layer: "session" in wm_memory.add/list/get/update/delete. FSRS-based eviction when capacity exceeded (no TTL).
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Add DashMap<String, MemoryEntry> or Arc<RwLock<HashMap>> in-memory store
- [x] #2 Implement layer: "session" in wm_memory.add/list/get/update/delete handlers
- [x] #3 FSRS-based eviction when capacity exceeds threshold
- [x] #4 Session memory is scoped to MCP server process lifetime
- [x] #5 Add tests: add + list in same session, entries isolated across sessions
<!-- AC:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Implemented: Added session_memory: DashMap<String, MemoryEntry> to EngineState. Changed memory_dir() to not error for "session". Added session branch to list/get/add/update/delete handlers. Added FSRS-based eviction (evict_lowest_fsrs using recency_boost at SESSION_CAPACITY=1000). All 148 tests pass.
<!-- SECTION:NOTES:END -->

