---
title: Make task cards clickable for detail view
id: b7fbf7
type: task
status: done
priority: medium
acceptance_criteria:
  - text: "Clicking a task card in tasks-view.component.ts opens a detail view, drawer, or dialog with full task info"
  - text: "Task cards are interactive (no longer static labels), with a visible affordance and click handling"
---

schema_version: 1
state: |-
  In tasks-view.component.ts, make task cards interactive so clicking them opens a detail view, drawer, or dialog with full task info. Currently they are static labels with no interaction.
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
