---
title: TD-11 Wire record validation into wm_validate
type: task
id: "wiki:tasks:td-11-wire-record-validation-into-wmvalidate"
status: done
priority: high
tags: [from-spec, spec:typed-decision-doc-format, engine, gap]
spec: specs/typed-decision-doc-format
acceptance_criteria:
  - text: "wm_validate.check runs the record-envelope validator on record-bearing pages"
  - text: "Malformed records (FR-5) are reported with page + field"
  - text: "Tests cover validate wiring"
relates_to:
  - {type: relates_to, target: wiki:specs:typed-decision-doc-format}
---

schema_version: 1
state: |-
  Follow-up from TD-06: `wm_validate.check` does not yet run `validate_record` on record bodies (FR-5/AC-3 gap). Wire the record-envelope validator into the validate tool so malformed records are caught.
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
