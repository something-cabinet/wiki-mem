---
title: Standardize page headers across all views
id: befdeb
type: task
status: done
priority: medium
tags: [ux, consistency, layout]
acceptance_criteria:
  - text: "All views (Graph, Settings, Search, Tasks, Pages, Memory) use a consistent header pattern based on the Graph header reference (bg-card, border-b, proper padding)"
  - text: "No view retains a divergent header pattern (plain h1 or standalone heading + button row)"
---

schema_version: 1
state: |-
  Each view has a different header pattern:
  - Graph: header bar with badges
  - Settings: heading + button row
  - Search/Tasks/Pages/Memory: plain h1
  Standardize to a consistent header pattern across all views. Use the Graph header as the reference (bg-card, border-b, proper padding).
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
