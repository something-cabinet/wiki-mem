---
title: SA-03 Tests + validation for all-types records
type: task
id: "wiki:tasks:sa-03-tests--validation-for-all-types-records"
status: todo
priority: high
tags: [from-spec, spec:structured-all-doc-types, testing]
spec: specs/structured-all-doc-types
acceptance_criteria:
  - text: "Tests: rule loading, memory recall, task description, search memory branch all read state"
  - text: "No regression in search/graph/validate; zero warnings"
---

schema_version: 1
state: |-
  Tests + validation for all-types records and consumer rewiring. Depends on SA-01/SA-02.
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
