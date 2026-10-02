---
id: wiki:patterns:task-subagents-for-delegation
title: 'Pattern: Use Task Subagents for Delegation'
type: pattern
tags: [pattern, workflow, delegation]
status: draft
relates_to:
  - {type: references, target: wiki:patterns:run-clippy-before-rust-reviewer}
---

schema_version: 1
state: |-
  id: wiki:patterns:task-subagents-for-delegation

  ## Problem

  Spawning separate Discord threads via `kimaki send --thread` for delegating work within the same project causes loss of control, untracked progress, and context switching.

  ## Solution

  Use the `task` tool to spawn subagents instead. Subagents run in their own context window, return results to the orchestrator, and keep the user in a single thread. The orchestrator can parallelize independent work by spawning multiple `task` subagents simultaneously.

  ```
  Orchestrator              task subagent
      │                          │
      ├── task("review X") ──────┤
      │                          ├── reads files
      │                          ├── runs checks
      │                          └── returns results
      │←──── results ────────────┤
      │                          │
      └── presents to user ──────┘
  ```

  ## When to Use

  - Delegating work within the same project
  - Parallelizing independent review/implementation tasks
  - Any time you need context isolation with results returned

  ## When Not to Use

  - The user explicitly asks for a separate thread
  - Work needs to happen in a different project repo
  - Sending a notification-only ping (use notification tools instead)

  ## Related

  - patterns/run-clippy-before-rust-reviewer
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
