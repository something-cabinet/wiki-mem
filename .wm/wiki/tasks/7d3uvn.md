---
title: CLI E2E Integration Tests
type: task
status: done
tags: [test, cli, integration]
priority: high
id: 7d3uvn
acceptance_criteria:
  - text: "wm-core/tests/cli_test.rs covers CLI smoke tests: init, create pages for all 7 types, search (keyword/semantic/hybrid), graph operations (neighbors/stats/path/subgraph), task board, time tracking, lint/validate, and index rebuild"
  - text: "All commands are exercised with the --json flag"
---

schema_version: 1
state: |-
  # CLI E2E Integration Tests

  > *Imported from Knowns task `7d3uvn`*

  # CLI E2E Integration Tests

  ## Description


  Create wm-core/tests/cli_test.rs with CLI smoke tests: init project, create pages (all 7 types), search (keyword/semantic/hybrid), graph operations (neighbors/stats/path/subgraph), task board, time tracking, lint/validate, index rebuild. Test with --json flag on all commands.


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
