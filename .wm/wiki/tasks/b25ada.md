---
title: 'GFX: Fix color legend oklch double-wrap'
id: b25ada
type: task
status: done
priority: high
tags:
- spec:graph-ui-fix
- bug
acceptance_criteria:
- text: buildPageTypes() reads --page-type-{key} CSS vars directly without re-wrapping values in oklch()
- text: Legend swatches render with visible (non-transparent) colors
- text: GraphColorService is used for color derivation where available
---

schema_version: 1
state: |-
  buildPageTypes() in graph-view.component.ts double-wraps oklch(oklch(...)) making legend swatches transparent. Fix: read --page-type-{key} directly, do NOT re-wrap in oklch(). Use GraphColorService if available.
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
