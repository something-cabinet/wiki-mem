---
title: Update wm-extract skill with core promotion and staleness check
type: task
tags:
- from-spec
- spec:core-page-type
status: done
priority: medium
acceptance_criteria:
- text: Step 7b exists in wm-extract after Step 7
  checked: false
- text: Step 7b checks if extraction qualifies as core (meta-project, foundational)
  checked: false
- text: 'Step 7b creates type: core pages with references edge when qualified'
  checked: false
- text: Staleness check scans existing core pages for stale references
  checked: false
- text: Staleness check reports suggestions (does not auto-update)
  checked: false
---

schema_version: 1
state: |-
  Add Step 7b (promote to core) after Step 7 (promote to critical). Add staleness check that scans existing core pages during extraction.
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
