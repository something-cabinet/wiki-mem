---
title: "MCPClient: replace fixed sleep with active readiness polling"
type: task
status: done
tags: [review-fix, test-infra]
priority: high
id: 9bkmxg
acceptance_criteria:
  - text: "MCPClient::start() no longer uses a fixed 500ms sleep — readiness is polled via retry initialize() with 100ms backoff"
  - text: "Startup succeeds within a 10s deadline on slow CI runners without flaky failures"
---

schema_version: 1
state: |-
  # MCPClient: replace fixed sleep with active readiness polling

  > *Imported from Knowns task `9bkmxg`*

  # MCPClient: replace fixed sleep with active readiness polling

  ## Description


  P1 from rust-reviewer. MCPClient::start() used fixed 500ms sleep. Replaced with retry initialize() with 100ms backoff and 10s deadline. Removes flaky startup on slow CI runners.


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
