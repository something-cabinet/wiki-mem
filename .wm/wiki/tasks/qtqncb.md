---
title: Add full doc CRUD tools (wm_doc.get/create/update/delete)
type: task
status: done
tags: [feature, docs, knowns-parity]
priority: high
id: qtqncb
acceptance_criteria:
  - text: "wm_doc.get exists and reads doc content by path"
  - text: "wm_doc.create, wm_doc.update, and wm_doc.delete exist to fully manage docs via MCP"
---

schema_version: 1
state: |-
  # Add full doc CRUD tools (wm_doc.get/create/update/delete)

  > *Imported from Knowns task `qtqncb`*

  # Add full doc CRUD tools (wm_doc.get/create/update/delete)

  ## Description


  WM only has wm_doc.list for .knowns/docs/. Need get, create, update, delete to fully manage docs via MCP. Currently users must use wm_page.* for wiki pages, but there's no bridge to Knowns docs.


  ## Acceptance Criteria

  - [x] #1 wm_doc.get exists (read doc content by path)
  - [x] #2 wm_doc.create exists (create new doc)
  - [x] #3 wm_doc.update exists (update existing doc)
  - [x] #4 wm_doc.delete exists (remove doc)
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
