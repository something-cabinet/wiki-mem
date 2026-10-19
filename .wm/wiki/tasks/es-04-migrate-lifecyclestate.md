---
title: ES-04 Migrate LifecycleState
type: task
id: "wiki:tasks:es-04-migrate-lifecyclestate"
status: done
priority: high
acceptance_criteria:
  - text: "LifecycleState owns skill_engine, write_channel and index_scheduler; no call site reads them off a sibling handle"
  - text: "Skill events, the serialized write channel and index scheduling behave unchanged (FR-3)"
  - text: "A unit test proves LifecycleState owns its state and does not reach into a sibling handle"
  - text: "cargo build, test and clippy --workspace are green with no new warnings"
tags: [from-spec, spec:engine-state-decomposition, refactor, wm-core, engine]
implementation_notes: |-
  Implemented in 03f4e4f. LifecycleState owns skill_engine, write_channel and index_scheduler privately;
  consumers use scan_skills, fire_skill_event, skill_tool_specs, write and submit_index.

  Flat pub fields 16 -> 13. Name was free (rg 'LifecycleState' returned nothing), so no spec amendment.

  Verified: cargo test --workspace run twice - exit=0 both times, 42 suites ok, 0 failed.
  Two code_index_watcher_test failures in an earlier run were the known 20s poll_until deadline under parallel
  load; the same tests pass 7/7 in 1.23s in isolation.
---

schema_version: 1
state: |-
  Fourth step of @wiki/specs/engine-state-decomposition (D-7). Migrate `LifecycleState`, owning the
  three long-lived subsystem handles `skill_engine`, `write_channel` and `index_scheduler`.

  These fields have no shared consumer contract beyond construction, so they move together. D-6 rejected
  folding `lsp` in here as a grouping on the loose commonality of long-lived subsystem; do not.
  Depends on ES-01.
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