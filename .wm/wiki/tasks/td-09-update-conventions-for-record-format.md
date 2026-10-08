---
title: TD-09 Update conventions for record format
type: task
id: "wiki:tasks:td-09-update-conventions-for-record-format"
status: done
priority: medium
tags: [from-spec, spec:typed-decision-doc-format, docs]
spec: specs/typed-decision-doc-format
acceptance_criteria:
  - text: "Conventions updated for the record format"
  - text: "Schema spec published and linked"
relates_to:
  - {type: relates_to, target: wiki:specs:typed-decision-doc-format}
---

schema_version: 1
state: |-
  Update conventions/docs for the typed-decision record format. Depends on TD-01. Non-destructive.
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
