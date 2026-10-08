---
title: Mark existing TUI wiki items superseded/cancelled
type: task
id: "wiki:tasks:mark-existing-tui-wiki-items-supersededcancelled"
status: done
priority: medium
tags: [from-spec, spec:remove-tui]
acceptance_criteria:
  - text: "AC-9: tui-polish spec marked superseded; tasks 75k8oh and 6lzncr marked cancelled, all linking to specs/remove-tui"
assignee: fixer
---

schema_version: 1
state: |-
  Per @wiki/specs/remove-tui FR-8: mark @wiki/specs/tui-polish-search-scrolling-pagination-tab-cycle-unicode as superseded, and @wiki/tasks/75k8oh and @wiki/tasks/6lzncr as cancelled, each linking to this spec via relates_to.
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
