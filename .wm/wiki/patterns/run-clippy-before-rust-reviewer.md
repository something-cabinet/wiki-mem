---
id: wiki:patterns:run-clippy-before-rust-reviewer
title: 'Pattern: Run Clippy Before Spawning rust-reviewer'
type: pattern
tags: [pattern, review, workflow]
status: draft
relates_to:
  - {type: references, target: wiki:patterns:task-subagents-for-delegation}
---

schema_version: 1
state: |-
  id: wiki:patterns:run-clippy-before-rust-reviewer

  ## Problem

  The `rust-reviewer` subagent often gets stuck for 30-60 seconds then fails, wasting time and breaking flow.

  ## Solution

  Before spawning `rust-reviewer`, run `cargo clippy -- -D warnings` yourself. If it fails, fix the pre-existing issues first — usually `#[allow(dead_code)]` on schema-only input struct fields in MCP tool handlers. Then spawn the reviewer. It will only check for NEW issues in the diff, which is fast and clean.

  ## When to Use

  - Any time you plan to delegate a code review to `rust-reviewer`
  - Before committing, to ensure clippy-clean state

  ## When Not to Use

  - If the workspace has no Rust code
  - If you're intentionally deferring clippy fixes

  ## Related

  - concepts/delegation-task-subagents-vs-separate-sessions
questions:
  - id: problem_kind
    type: choice
    instructions: What kind of problem does this pattern solve?
    options:
    - architecture
    - api-design
    - data-model
    - error-handling
    - performance
    - testing
    - ui
    - tooling
    - workflow
  - id: preconditions_required
    type: noul
    instructions: This pattern requires specific preconditions to be met.
  - id: complexity
    type: score
    instructions: How complex is applying this pattern?
    levels:
    - trivial
    - simple
    - moderate
    - complex
  - id: language_specific
    type: noul
    instructions: This pattern is specific to a programming language.
answers: {}
