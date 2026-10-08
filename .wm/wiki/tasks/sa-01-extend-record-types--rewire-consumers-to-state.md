---
title: SA-01 Extend record types + rewire consumers to state
type: task
id: "wiki:tasks:sa-01-extend-record-types--rewire-consumers-to-state"
status: todo
priority: high
tags: [from-spec, spec:structured-all-doc-types, engine, p0]
spec: specs/structured-all-doc-types
acceptance_criteria:
  - text: "is_record_bearing + canonical_questions cover rule/core/memory/task/spec/note"
  - text: "wm_page.get returns state as content for record pages"
  - text: "memory read, task description, search memory branch read state"
  - text: "rule loading still yields the rule text"
---

schema_version: 1
state: |-
  Extend record types + per-type question sets to all page types, and rewire prose consumers (wm_page.get content, memory read, task description, search memory branch) to read `state`. Per specs/structured-all-doc-types.
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
