---
title: "TUI: Dashboard Scrolling + Search Polish"
type: task
status: cancelled
tags: [tui, ratatui, ux]
priority: medium
id: 6lzncr
acceptance_criteria:
  - text: "Dashboard replaces Paragraph with Scrollbar+List to handle more than 50 pages"
  - text: "Search supports Ctrl+V paste in raw mode and cycles results with Enter for preview"
  - text: "Tab/Shift+Tab cycles tabs, ? shows a help overlay with all bindings, and the graph center node is selectable via search"
relates_to:
  - {type: superseded_by, target: wiki:specs:remove-tui}
---

schema_version: 1
state: |-
  # TUI: Dashboard Scrolling + Search Polish

  > *Imported from Knowns task `6lzncr`*

  # TUI: Dashboard Scrolling + Search Polish

  ## Description


  Fix Ratatui TUI issues: (1) Dashboard — replace Paragraph with Scrollbar+List for >50 pages, (2) Search — support Ctrl+V paste via raw mode, cycle results with Enter to preview, (3) Graph — allow selecting center node via search, (4) Tab cycling — Tab/Shift+Tab to cycle tabs, (5) Help overlay — ? key shows all bindings.


  ## Acceptance Criteria
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
