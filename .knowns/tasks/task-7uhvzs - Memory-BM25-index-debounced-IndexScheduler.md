---
id: 7uhvzs
title: Memory BM25 index + debounced IndexScheduler
status: done
priority: high
labels:
  - from-spec
  - go-mode
createdAt: '2026-07-07T04:51:11.661Z'
updatedAt: '2026-07-07T04:57:45.055Z'
timeSpent: 0
---
# Memory BM25 index + debounced IndexScheduler

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Add memory indexing and debounced scheduler:
1. Read .wm/memory/*.json into MemoryEntry structs
2. IndexMemory in EngineState (separate ArcSwap)
3. build_memory_index(), search_memory()
4. Replace AtomicBool stale_flag with IndexScheduler (500ms debounce)
5. Extend wm_index.rebuild for both indexes
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
<!-- AC:END -->

