---
title: ES-03 Migrate MemoryState
type: task
id: "wiki:tasks:es-03-migrate-memorystate"
status: done
priority: high
acceptance_criteria:
  - text: "MemoryState owns session_memory; no call site reads the map off a sibling handle"
  - text: "memory_gate and the wm_memory tool surface are behaviourally unchanged (FR-3)"
  - text: "A unit test proves MemoryState owns its state and does not reach into a sibling handle"
  - text: "cargo build, test and clippy --workspace are green with no new warnings"
tags: [from-spec, spec:engine-state-decomposition, refactor, wm-core, memory]
implementation_notes: |-
  Implemented in b20362a. MemoryState owns a private DashMap behind count/insert/snapshot/remove;
  memory_gate and tools/memory.rs use methods; MemoryEntry gained Clone for owned snapshots.

  Flat pub fields 17 -> 16 (19 minus audit, source_registry, session_memory). Name was free:
  rg 'MemoryState' returned nothing before the edit, so spec D-5 needs no amendment.

  Verified: cargo test -p wm-core = 205 lib tests plus all integration suites green; cargo test -p wm-tools
  all suites green; cargo test --workspace exit=0, 42 suites ok, 0 failed.
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