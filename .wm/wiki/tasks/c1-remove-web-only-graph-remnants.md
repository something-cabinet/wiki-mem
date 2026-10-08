---
title: C1 Remove web-only graph remnants
type: task
id: "wiki:tasks:c1-remove-web-only-graph-remnants"
status: todo
priority: medium
tags: [cleanup, graph, web-remnant]
acceptance_criteria:
  - text: "wm_graph.full removed (action, handler, registration, CLI refs)"
  - text: "Tests + agent instructions no longer reference wm_graph.full"
  - text: "Stale 'HTTP graph routes' doc line fixed"
  - text: "Dead cached_db_is_fresh stub removed or justified"
  - text: "cargo check/clippy/tests clean; grep-clean"
---

schema_version: 1
state: |-
  Remove web-only graph remnants after wm-web retirement: the `wm_graph.full` tool (viz contract), stale doc line, and the dead `cached_db_is_fresh` no-op. Per the graph-usage recon.
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
