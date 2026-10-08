---
title: Add wm_template.create tool
type: task
status: done
tags: [feature, templates, knowns-parity]
priority: medium
id: o26wkw
acceptance_criteria:
  - text: "wm_template.create accepts name, description, and content with {{variable}} placeholders"
  - text: "Template written to .wm/templates/<name>.json and appears in wm_template.list"
---

schema_version: 1
state: |-
  # Add wm_template.create tool

  > *Imported from Knowns task `o26wkw`*

  # Add wm_template.create tool

  ## Description


  WM's template system supports list/get/run but not create. Templates must be manually placed as JSON files in .wm/templates/. Add a create tool that accepts name, description, content with {{variable}} placeholders.


  ## Acceptance Criteria

  - [x] #1 wm_template.create accepts name, description, content
  - [x] #2 Template written to .wm/templates/<name>.json
  - [x] #3 Created template appears in wm_template.list
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
