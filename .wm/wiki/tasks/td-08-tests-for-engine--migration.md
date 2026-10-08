---
title: TD-08 Tests for engine + migration
type: task
id: "wiki:tasks:td-08-tests-for-engine--migration"
status: done
priority: high
tags: [from-spec, spec:typed-decision-doc-format, testing]
spec: specs/typed-decision-doc-format
acceptance_criteria:
  - text: "Tests for parser, validation, round-trip, migration idempotency"
  - text: "No regression in page read/graph/search"
relates_to:
  - {type: relates_to, target: wiki:specs:typed-decision-doc-format}
---

schema_version: 1
state: |-
  Tests covering TD-03..TD-06 and AC-3/4/8/9. Depends on engine + migration.
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
