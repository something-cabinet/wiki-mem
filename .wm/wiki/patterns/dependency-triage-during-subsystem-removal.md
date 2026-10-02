---
title: 'Pattern: Dependency Triage During Subsystem Removal'
type: pattern
id: "wiki:patterns:dependency-triage-during-subsystem-removal"
status: draft
tags: [pattern, cleanup, dependencies, rust]
relates_to:
  - {type: references, target: wiki:tasks:remove-tui-code-and-ratatui-dependency-from-wm-cli}
---

schema_version: 1
state: |-
  ## Problem

  When removing a subsystem (TUI, legacy module, old integration), its dependencies must be triaged individually — some are subsystem-only, others are shared with surviving features. Removing shared dependencies breaks unrelated flows.

  ## Solution

  Before deleting a subsystem, classify each of its dependencies by actual usage across the codebase:

  1. **Subsystem-only** (e.g. `ratatui` used only in `tui.rs`) → remove from Cargo.toml; `cargo build` prunes it from Cargo.lock.
  2. **Shared with surviving features** (e.g. `is-terminal` used by the wizard's stdin detection, `dialoguer`/`indicatif` used by `wm init`/spinners) → keep.
  3. Verify with `rg <dep> <crate-src>` — grep the source, not just Cargo.toml, to find every call site before deciding.

  ## When to Use

  - Removing a feature, module, or integration from a Rust workspace
  - Any "delete X" task where X has dependencies

  ## When Not to Use

  - Adding features (no removal to triage)
  - Removing a leaf dependency with no other consumers

  ## Related

  - @wiki/tasks/remove-tui-code-and-ratatui-dependency-from-wm-cli
  - @wiki/specs/remove-tui
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
