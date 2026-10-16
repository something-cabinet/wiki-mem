---
title: ES-06 Migrate GraphState
type: task
id: "wiki:tasks:es-06-migrate-graphstate"
status: todo
priority: high
acceptance_criteria:
  - text: "GraphState owns graph; no call site reads the snapshot off a sibling handle"
  - text: "graph/ reaches section_corpus and bm25_index only through explicit methods on IndexState (scenario 2)"
  - text: "ArcSwap is preserved for the graph snapshot; no new lock on the read path (NFR-1)"
  - text: "A unit test proves GraphState owns its state and does not reach into a sibling handle"
  - text: "cargo build, test and clippy --workspace are green with no new warnings"
tags: [from-spec, spec:engine-state-decomposition, refactor, wm-core, graph]
---

schema_version: 1
state: |-
  Sixth step of @wiki/specs/engine-state-decomposition (D-7). Migrate `GraphState`, owning `graph`,
  the `ArcSwap` snapshot plus its node index.

  `graph/` currently imports `engine`, `parser` and `search`; after this step it must reach the corpus and the
  index through methods on `IndexState` rather than loading their fields directly, which is AC-3 and scenario 2
  of the spec. NFR-1 still binds: no new lock on the graph read path. Depends on ES-05.
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
