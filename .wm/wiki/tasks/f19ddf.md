---
title: 'GFX: Instantiate ResizeObserver in canvas directive'
id: f19ddf
type: task
status: done
priority: medium
tags:
- spec:graph-ui-fix
- bug
acceptance_criteria:
- text: ResizeObserver is instantiated in ngAfterViewInit and observes canvas.parentElement
- text: The manual this.resize() call at line 63 is removed, and the canvas resizes when the sidebar collapses or the window changes
---

schema_version: 1
state: |-
  canvas-graph.directive.ts declares ResizeObserver at line 37 but never instantiates it. Add new ResizeObserver(() => this.resize()) in ngAfterViewInit, observe canvas.parentElement. Remove the manual this.resize() call at line 63.
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
