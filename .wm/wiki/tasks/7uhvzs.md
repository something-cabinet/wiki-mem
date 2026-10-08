---
title: Memory BM25 index + debounced IndexScheduler
type: task
status: done
tags: [from-spec, go-mode]
priority: high
id: 7uhvzs
acceptance_criteria:
  - text: ".wm/memory/*.json files are read into MemoryEntry structs and indexed via IndexMemory in EngineState (separate ArcSwap)"
  - text: "build_memory_index() and search_memory() are implemented and functional"
  - text: "AtomicBool stale_flag replaced with an IndexScheduler using 500ms debounce, and wm_index.rebuild rebuilds both the page and memory indexes"
---

schema_version: 1
state: |-
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
questions:
  - id: work_kind
    type: choice
    instructions: What kind of work is this task?
    options:
    - feature
    - bugfix
    - refactor
    - docs
    - test
    - chore
    - migration
  - id: priority
    type: choice
    instructions: What priority is this task?
    options:
    - low
    - medium
    - high
    - urgent
  - id: needs_spec
    type: noul
    instructions: This task depends on a spec.
  - id: has_ac
    type: noul
    instructions: This task has at least one acceptance criterion.
answers: {}
