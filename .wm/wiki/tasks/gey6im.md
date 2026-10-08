---
title: Implement code intelligence MCP tools
type: task
status: done
tags: [feature, code-intelligence, knowns-parity]
priority: high
id: gey6im
acceptance_criteria:
  - text: "Code search tool exists supporting search by symbol name and text pattern"
  - text: "Symbol lookup tool exists finding definitions and references"
  - text: "Dependency graph tool exists showing file imports and module dependencies"
---

schema_version: 1
state: |-
  # Implement code intelligence MCP tools

  > *Imported from Knowns task `gey6im`*

  # Implement code intelligence MCP tools

  ## Description


  WM has no code intelligence. Knowns provides knowns_code with AST-based search, symbol lookup, dependency graphs. WM needs equivalent tools for code search across the codebase. This is the largest feature gap.


  ## Acceptance Criteria

  - [x] #1 Code search tool exists (search by symbol name, text pattern)
  - [x] #2 Symbol lookup tool exists (find definitions, references)
  - [x] #3 Dependency graph tool exists (file imports, module deps)
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
