---
title: GWL-02 Scope write model + closed shareable set
type: task
id: "wiki:tasks:gwl-02-scope-write-model-and-shareable-set"
status: cancelled
tags: [from-spec, spec:global-wiki-layer, mcp, validation]
acceptance_criteria:
  - text: scope=global lands in $HOME/.wm and is immediately searchable in the same process; the response reports the actual destination path, never an echo of the request
  - text: scope=global with task/spec/core returns a validation error naming the closed set and writes zero bytes
  - text: A hand-placed non-shareable page in the global root is excluded from the index with a lint warning
priority: high
---

schema_version: 1
state: |-
  Implements D4 and D5 with AC-3 and AC-4 of @wiki/specs/global-wiki-layer

  - Page writes take an optional `scope` (default `project`). Writing a page whose effective copy is global writes the global file — one file, one truth, therefore no drift by construction.
  - Response reports the **actual destination path**, never an echo of the request (the `layer` echo defect in `memory.rs:370` is the precedent to avoid).
  - Closed shareable set enforced twice, because `$HOME/.wm/wiki/` is hand-editable: at the write path (`scope=global` with `task`/`spec`/`core` → validation error naming the set, zero bytes written) and at index merge (non-shareable pages physically in the global root are excluded and surfaced as a lint warning).
  - Shareable: `rule, pattern, concept, howto, reference, decision, memory`. Project-local: `task, spec, core, index.md`.
  - Write-path refresh must make a global write immediately searchable in the same process.
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