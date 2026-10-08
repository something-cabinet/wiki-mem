---
title: Fix core README staleness (wm-cli naming, wm serve, requirements)
type: task
id: wiki:tasks:fix-core-readme-staleness
status: done
tags:
- docs
- readme
- cli-naming
- staleness
time_started: 2026-08-06T03:24:21.417041+00:00
implementation_notes: Plan presented (align wm-cli naming, replace wm serve with wm-cli web, npm-primary requirements). Declined by user — not implementing. Closing as done per user instruction.
time_spent: 0h 3m
acceptance_criteria:
- text: Core wiki README aligned with repo README naming (wm-cli, wm-cli web), or change explicitly declined by user with outcome documented
  checked: false
---

schema_version: 1
state: |-
  Align the wiki core README page (wiki:core:README) with the repo README.md already fixed in commit d393861.

  Known staleness (from extract session):
  - Quick Start uses `wm init` / `wm mcp` / `wm` — binary is `wm-cli`
  - CLI Commands table uses `wm init`, `wm mcp`, `wm serve`, etc. — `wm serve` was renamed; actual command surface is `wm-cli web` (verified against wm-cli --help in the fix-readme task)
  - Setup Workflow uses `wm setup opencode`, `wm model download`, `wm index embed` — should be `wm-cli`
  - Requirements says "Rust toolchain 1.75+" only — npm install is now primary

  Out of scope: other wiki pages, repo README.md, docs/README.md.
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
