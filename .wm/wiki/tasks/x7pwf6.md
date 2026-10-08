---
title: Fix .ok() swallows, assert_contains! macro, Windows kill, unused import
type: task
status: done
tags: [review-fix, test-infra]
priority: medium
id: x7pwf6
acceptance_criteria:
  - text: "The 9 .ok() calls in setup_test_project() are replaced with .expect() calls carrying descriptive messages"
  - text: "The assert_contains! macro binds $haystack to locals so it is not double-evaluated"
  - text: "The Windows-only taskkill is replaced with a cross-platform kill_process() helper and the unused Read module-level import is removed"
---

schema_version: 1
state: |-
  # Fix .ok() swallows, assert_contains! macro, Windows kill, unused import

  > *Imported from Knowns task `x7pwf6`*

  # Fix .ok() swallows, assert_contains! macro, Windows kill, unused import

  ## Description


  P2 items from rust-reviewer:
  - 9 .ok() calls in setup_test_project() → .expect() with messages
  - assert_contains! macro double-evaluated $haystack → bind to locals
  - run_cli_with_timeout used Windows-only taskkill → cross-platform kill_process() helper
  - Removed unused Read from module-level import

  All done.


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
