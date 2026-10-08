---
title: IX-03 Embedding integrity + degraded surfacing
type: task
id: "wiki:tasks:ix-03-embedding-integrity--degraded-surfacing"
status: done
priority: high
tags: [from-spec, spec:reinforce-indexing-flow, robustness]
spec: specs/reinforce-indexing-flow
acceptance_criteria:
  - text: "Persisted model/version validated on load; mismatch forces re-embed or clear"
  - text: "degraded + actionable reason surfaced in query/retrieve/index status"
---

schema_version: 1
state: |-
  Embedding integrity: validate persisted model/version on load, force re-embed/clear on mismatch, surface degraded reason everywhere. Per specs/reinforce-indexing-flow FR-3.
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
