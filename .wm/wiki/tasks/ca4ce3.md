---
title: 'GFX: Tune fjadra centering force for degree-0 nodes'
id: ca4ce3
type: task
status: done
priority: medium
tags:
- spec:graph-ui-fix
- graph
- layout
acceptance_criteria:
- text: Degree-0 nodes stay within the viewport after the fjadra layout settles
- text: Center force strength is tuned in the Rust layout command handler so the default (too-weak) strength is no longer used
---

schema_version: 1
state: |-
  Tune fjadra Center force strength so degree-0 nodes stay within viewport. Currently Center::new() with default strength may be too weak. Adjust parameters in the Rust layout command handler.
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
