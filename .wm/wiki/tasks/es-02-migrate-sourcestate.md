---
title: ES-02 Migrate SourceState
type: task
id: "wiki:tasks:es-02-migrate-sourcestate"
status: todo
priority: high
acceptance_criteria:
  - text: "SourceState owns source_registry; no call site reads the registry off a sibling handle or off a flat EngineState field"
  - text: "source_service.rs reaches the registry only through methods on SourceState"
  - text: "A unit test proves SourceState owns its state and does not reach into a sibling handle"
  - text: "cargo build, test and clippy --workspace are green with no new warnings"
tags: [from-spec, spec:engine-state-decomposition, refactor, wm-core, source]
---

schema_version: 1
state: |-
  Second step of @wiki/specs/engine-state-decomposition (D-7). Migrate `SourceState`, owning
  `source_registry` (15 references, 11 of them in `source_service.rs`).

  The registry is already reached through service methods; the work is moving the field behind the handle and
  deleting the direct field reads. Depends on the ES-01 scaffold.
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
