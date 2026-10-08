---
title: Update README to remove TUI references
type: task
id: "wiki:tasks:update-readme-to-remove-tui-references"
status: done
priority: medium
tags: [from-spec, spec:remove-tui]
acceptance_criteria:
  - text: "AC-8: README.md no longer mentions the TUI"
assignee: fixer
---

schema_version: 1
state: |-
  Update README.md per @wiki/specs/remove-tui FR-7. Remove TUI references at lines 3, 20, 102, 124 and the `wm-cli tui` usage row.
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
