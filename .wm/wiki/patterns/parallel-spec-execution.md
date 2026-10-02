---
id: wiki:patterns:parallel-spec-execution
title: 'Pattern: Parallel Spec Execution'
type: pattern
status: reviewed
tags: [pattern, workflow, parallelism, specs]
relates_to:
  - {type: references, target: wiki:specs:one-struct-per-file}
  - {type: references, target: wiki:specs:wm-spec-typed-pages}
  - {type: references, target: wiki:tasks:aa0388}
---

schema_version: 1
state: |-
  id: wiki:patterns:parallel-spec-execution

  ## Problem
  Executing multiple independent specs sequentially wastes time. Mechanical refactors and workflow changes don't depend on each other but are done one after another.

  ## Solution
  Use `kimaki send` to spawn sub-sessions for independent specs. Each sub-session gets:
  - A complete, self-contained prompt with the spec content, acceptance criteria, and verification steps
  - Its own thread, working directory, and agent
  - Clear pass/fail criteria (cargo build, cargo test)

  The main session continues with one spec while the sub-session handles another in parallel.

  ## When to Use
  - Specs that touch different modules (e.g., file structure refactor + SKILL.md changes)
  - Mechanical refactors that have clear verification criteria
  - Tasks that can fail independently without blocking each other

  ## When Not to Use
  - Specs that modify the same files (merge conflicts guaranteed)
  - Sequential dependencies (spec B builds on spec A's output)
  - When sub-session context overhead > doing it yourself

  ## Related
  - @wiki/specs/one-struct-per-file
  - @wiki/specs/wm-spec-typed-pages
  - @wiki/tasks/aa0388
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
