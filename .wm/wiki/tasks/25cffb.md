---
title: 'GFX: Move spacing/zoom controls to floating canvas toolbar'
id: 25cffb
type: task
status: done
priority: medium
tags:
- spec:graph-ui-fix
- ux
acceptance_criteria:
- text: Spacing slider and zoom controls move into a floating toolbar at bottom-center of the graph canvas
- text: Graph header is reduced to title plus stats badges, consistent with all other views
---

schema_version: 1
state: |-
  Move spacing slider out of graph header into a floating toolbar on the canvas (bottom-center). Also consolidate zoom controls into the same toolbar. Header becomes title + stats badges only — consistent with all other views.
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
