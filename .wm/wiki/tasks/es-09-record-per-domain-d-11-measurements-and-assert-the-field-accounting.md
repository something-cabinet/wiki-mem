---
title: ES-09 Record per-domain D-11 measurements and assert the field accounting
type: task
id: "wiki:tasks:es-09-record-per-domain-d-11-measurements-and-assert-the-field-accounting"
status: done
priority: medium
acceptance_criteria:
  - text: "The spec records per domain whether it clears D-11 (a)/(b)/(c), with the measurement (AC-5)"
  - text: "A checked-in grep count asserts the 21 original fields are each accounted for exactly once - 20 owned by handles, 1 moved to wm-tools (AC-6)"
  - text: "EngineState exposes no flat public fields for the migrated domains (AC-1)"
tags: [from-spec, spec:engine-state-decomposition, docs, verification]
implementation_notes: |-
  Implemented in the following commits. AC-6 is asserted by test_engine_state_fields_are_accounted_for_once in
  engine_state_mediator.rs - it parses its own source and fails when the declared field set drifts; verified to
  fail both by adding and by removing a field.

  AC-5 recorded in the spec as an Outcome section: no handle earns a crate. Line counts 124-243 (all fail the
  1500 mass bar by 6x+), none pulls rmcp/tree-sitter/ort, and wm-cli is the only product binary so (c) fails
  everywhere. D-3 is settled by measurement.

  Recorded honestly rather than claimed: AC-3 is 0 -> 0 (page/graph/search edges were already closed by ES-05
  and ES-06), and page_contents/content_hashes are dead - PageState's eight methods have no product caller.
  AuditState::started_at() has no reader either.
---

schema_version: 1
state: |-
  Closes AC-5 and AC-6 of @wiki/specs/engine-state-decomposition once the nine handles exist.

  AC-5: record, per domain, whether it now clears the D-11 rubric of mass, dependency-extraction value or
  multi-consumer, together with the measurement that decides it. Do not assume a crate is earned; the answer
  may be that none is, which is a valid recorded outcome.

  AC-6: assert the accounting by a checked-in grep count rather than prose - all 21 original fields are
  accounted for exactly once, 20 owned by the nine handles and 1 moved to wm-tools.
  Depends on ES-07 and ES-08.
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