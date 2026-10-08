---
title: Add --page-type-core CSS tokens and graph inference for core/ directory
type: task
tags:
- from-spec
- spec:core-page-type
status: done
priority: high
acceptance_criteria:
- text: --page-type-core CSS token exists in light and dark themes
  checked: false
- text: Files at .wm/wiki/core/*.md auto-resolve to PageType::Core
  checked: false
---

schema_version: 1
state: |-
  Add --page-type-core CSS custom property in light/dark themes. Add core/ directory inference mapping to PageType::Core in graph.rs.
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
