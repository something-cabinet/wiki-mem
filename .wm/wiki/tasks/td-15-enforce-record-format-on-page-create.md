---
title: TD-15 Enforce record format on page create
type: task
id: "wiki:tasks:td-15-enforce-record-format-on-page-create"
status: done
priority: high
tags: [from-review, spec:typed-decision-doc-format, p1, engine]
spec: specs/typed-decision-doc-format
acceptance_criteria:
  - text: "Creating a record-bearing page emits a valid record body (or rejects prose) per spec D-A"
  - text: "wm_validate flags prose bodies on record-bearing types (strict mode or by default)"
  - text: "Tests cover create + validate"
---

schema_version: 1
state: |-
  Review P1-3: the record format is not enforced for newly created pages (wm_decision.create writes prose; validation is opt-in). Make create emit a record (or reject prose) and make validation catch prose record-bearing pages.
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
