---
title: ES-03 Migrate MemoryState
type: task
id: "wiki:tasks:es-03-migrate-memorystate"
status: todo
priority: high
acceptance_criteria:
  - text: "MemoryState owns session_memory; no call site reads the map off a sibling handle"
  - text: "memory_gate and the wm_memory tool surface are behaviourally unchanged (FR-3)"
  - text: "A unit test proves MemoryState owns its state and does not reach into a sibling handle"
  - text: "cargo build, test and clippy --workspace are green with no new warnings"
tags: [from-spec, spec:engine-state-decomposition, refactor, wm-core, memory]
---

schema_version: 1
state: |-
  Third step of @wiki/specs/engine-state-decomposition (D-7). Migrate `MemoryState`, owning
  `session_memory`, the session-scoped map of memory entries (7 references).

  Keep the memory_gate read path unchanged; only the ownership of the map moves. Depends on ES-01.
  Part of @wiki/specs/engine-state-decomposition
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
