---
title: Update wm-init skill for dynamic core page discovery
type: task
tags:
- from-spec
- spec:core-page-type
status: done
priority: medium
acceptance_criteria:
- text: 'wm-init Step 3 dynamically discovers type: core pages'
  checked: false
- text: README still loaded explicitly as project intro
  checked: false
- text: Core pages listed in session context summary
  checked: false
implementation_notes: 'Extracted decision: @wiki/decisions/dynamic-core-discovery-over-hardcoded-ids'
---

schema_version: 1
state: |-
  Update Step 3 of wm-init: read README, then dynamically discover all type: core pages via wm_page.list. Add core pages to session context summary.
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
