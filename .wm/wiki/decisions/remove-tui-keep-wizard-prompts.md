---
title: 'Decision: Remove TUI, Keep Wizard Prompts'
type: decision
id: "wiki:decisions:remove-tui-keep-wizard-prompts"
status: approved
tags: [decision, tui, cli, cleanup]
relates_to:
  - {type: implements, target: wiki:specs:remove-tui}
---

schema_version: 1
state: |-
  ## Context

  wm-cli shipped a full-screen ratatui TUI (`tui.rs`, ~29KB) launched via the `--tui` flag, the `wm tui` subcommand, and auto-launch when run with no args on a terminal. The project's primary interfaces are CLI commands, the wm-server HTTP daemon, and the wm-web Angular UI. The TUI was legacy surface with its own polish backlog (@wiki/specs/tui-polish-search-scrolling-pagination-tab-cycle-unicode).

  ## Decision

  Remove the ratatui TUI entirely. Keep the dialoguer/indicatif interactive wizard prompts used by `wm init`/`wm setup`/spinners — they are a separate concern from the TUI. `wm` with no args now prints help and exits 0.

  ## Rationale

  The TUI duplicated interactive surface already covered by the web UI and CLI, and carried a maintenance backlog. The wizard prompts are used by non-TUI flows and would break `wm init`/`wm setup` if removed. No-args → help is the standard CLI convention.

  ## Consequences

  - `wm tui` and `wm --tui` are rejected (exit 2)
  - `wm` with no args prints help, exits 0
  - `ratatui` removed from Cargo.toml/Cargo.lock
  - tui-polish spec marked superseded; TUI tasks cancelled
  - Future interactive work targets the web UI, not a terminal TUI

  ## Related

  - @wiki/specs/remove-tui
  - @wiki/tasks/remove-tui-code-and-ratatui-dependency-from-wm-cli
questions:
  - id: outcome
    type: choice
    instructions: What is the recorded outcome of this decision?
    options:
    - adopted
    - rejected
    - deferred
    - superseded
    - abandoned
  - id: reversibility
    type: noul
    instructions: The decision can be reversed cheaply without data migration or cross-module breakage.
  - id: confidence
    type: score
    instructions: How strong is the recorded justification for the selected outcome?
    levels:
    - low
    - medium
    - high
  - id: impact
    type: choice
    instructions: How wide is the blast radius of this decision?
    options:
    - local
    - component
    - system
    - project-wide
answers: {}
