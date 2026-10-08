---
title: TD-14 Fix gliner backend silent missing-task degradation
type: task
id: "wiki:tasks:td-14-fix-gliner-backend-silent-missing-task-degradation"
status: done
priority: high
tags: [from-review, spec:typed-decision-doc-format, p1, runtime]
spec: specs/typed-decision-doc-format
acceptance_criteria:
  - text: "A missing task/label result from the backend is an error, not a fabricated labels[0] p=0.0"
  - text: "DecisionRuntime's MissingTaskResult guard can fire for the real backend"
  - text: "Test covers the missing-task case"
---

schema_version: 1
state: |-
  Review P1-2: gliner backend degrades a missing task result to labels[0] p=0.0 (silent wrong answer). Return an error instead.
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
