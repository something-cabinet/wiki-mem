---
title: TD-04 Record validation rules
type: task
id: "wiki:tasks:td-04-record-validation-rules"
status: done
priority: high
tags: [from-spec, spec:typed-decision-doc-format, engine]
spec: specs/typed-decision-doc-format
acceptance_criteria:
  - text: "FR-5 malformed cases each produce a deterministic error"
  - text: "Tests cover each malformed case"
relates_to:
  - {type: relates_to, target: wiki:specs:typed-decision-doc-format}
---

schema_version: 1
state: |-
  Implement FR-5 record validation: missing state, malformed questions, choice <2 options, score out of 2-10, answer key/value mismatch. Depends on TD-01/TD-03.
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
