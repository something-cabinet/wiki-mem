---
title: Memory BM25 index + debounced IndexScheduler
type: task
status: done
tags: [from-spec, go-mode]
priority: high
id: 7uhvzs
---

# Memory BM25 index + debounced IndexScheduler

> *Imported from Knowns task `7uhvzs`*

# Memory BM25 index + debounced IndexScheduler

## Description


Add memory indexing and debounced scheduler:
1. Read .wm/memory/*.json into MemoryEntry structs
2. IndexMemory in EngineState (separate ArcSwap)
3. build_memory_index(), search_memory()
4. Replace AtomicBool stale_flag with IndexScheduler (500ms debounce)
5. Extend wm_index.rebuild for both indexes


## Acceptance Criteria
