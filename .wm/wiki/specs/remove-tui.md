---
title: Remove TUI — Delete Ratatui TUI from wm-cli
type: spec
id: "wiki:specs:remove-tui"
status: approved
tags: [spec, tui, cli, cleanup, ratatui, approved]
---

schema_version: 1
state: |-
  ## Overview

  `wm-cli` ships a full-screen interactive TUI built on **ratatui** (`apps/wm-cli/src/tui.rs`, ~29KB). It is launched three ways: the `--tui` global flag, the `wm tui` subcommand, and automatically when `wm` runs with no subcommand on a terminal. The TUI is a legacy interactive surface — the project's primary interfaces are the CLI commands, the HTTP daemon (`wm-server`), and the web UI (`wm-web`). This spec removes the ratatui TUI entirely.

  The interactive **wizard prompts** (`dialoguer` for `wm init`/`wm setup` confirmations and `indicatif` spinners) are a separate concern and are **kept** — they are used by non-TUI CLI flows.

  ## Locked Decisions

  - **D1 — Remove only the ratatui TUI**: delete `apps/wm-cli/src/tui.rs`, the `--tui` global flag, the `tui` subcommand, and the auto-launch path. Keep the `dialoguer`/`indicatif` wizard prompts used by `wm init`, `wm setup`, and spinners.
  - **D2 — `wm` with no args prints help and exits 0**: the auto-launch fallback is replaced by printing usage/help and exiting successfully.
  - **D3 — Docs + wiki cleanup**: update `README.md` (4 TUI references) and mark the existing TUI wiki spec/tasks as superseded/cancelled.

  ## Requirements

  ### Functional Requirements

  - **FR-1**: Delete the TUI module `apps/wm-cli/src/tui.rs` and its `mod tui;` declaration (`apps/wm-cli/src/main.rs:22`).
  - **FR-2**: Remove the `--tui` global flag (`apps/wm-cli/src/main.rs:33-35`).
  - **FR-3**: Remove the `Tui` subcommand variant (`apps/wm-cli/src/main.rs:107`) and its dispatch arm (`apps/wm-cli/src/main.rs:1618-1623`).
  - **FR-4**: Remove the auto-launch block (`apps/wm-cli/src/main.rs:1203-1206`) that starts the TUI when no subcommand is given on a terminal.
  - **FR-5**: Change the no-command path (`apps/wm-cli/src/main.rs:1208-1214`) to print help/usage and exit 0 instead of the current message that references the TUI.
  - **FR-6**: Remove the `ratatui = "0.30"` dependency from `apps/wm-cli/Cargo.toml`. Keep `is-terminal`, `dialoguer`, and `indicatif` — they are used by the wizard flows (`is_terminal` at `main.rs:1349,1468`; `dialoguer` at `main.rs:2`; `indicatif` at `main.rs:2952`).
  - **FR-7**: Update `README.md` to remove TUI references (lines 3, 20, 102, 124) and the `wm-cli tui` usage row.
  - **FR-8**: Mark the existing TUI wiki items superseded/cancelled: @wiki/specs/tui-polish-search-scrolling-pagination-tab-cycle-unicode (superseded), @wiki/tasks/75k8oh and @wiki/tasks/6lzncr (cancelled), each linking to this spec.

  ### Non-Functional Requirements

  - **NFR-1**: No dead code left behind — the TUI module and all call sites are deleted, not commented out; no `#[allow(dead_code)]`.
  - **NFR-2**: Zero compiler warnings — `cargo build` and `cargo clippy` clean.
  - **NFR-3**: No `ratatui` reference remains in the workspace (code, Cargo.toml, or Cargo.lock after `cargo update`).

  ## Acceptance Criteria

  - [ ] AC-1: `wm --tui` is rejected as an unknown flag.
  - [ ] AC-2: `wm tui` is not a recognized command (`wm --help` omits it; invoking it errors).
  - [ ] AC-3: `wm` with no args prints help/usage and exits 0 (no TUI launch, no TUI mention).
  - [ ] AC-4: `apps/wm-cli/src/tui.rs` no longer exists and no `mod tui` / `run_tui` reference remains.
  - [ ] AC-5: `ratatui` is absent from `apps/wm-cli/Cargo.toml` and from `Cargo.lock`.
  - [ ] AC-6: `cargo build` and `cargo clippy` pass with zero warnings.
  - [ ] AC-7: `wm init` and `wm setup` still show interactive prompts/spinners (dialoguer/indicatif preserved).
  - [ ] AC-8: `README.md` no longer mentions the TUI.
  - [ ] AC-9: @wiki/specs/tui-polish-search-scrolling-pagination-tab-cycle-unicode is marked superseded; @wiki/tasks/75k8oh and @wiki/tasks/6lzncr are marked cancelled, all linking to this spec.

  ## Scenarios

  ### Scenario 1: No-command invocation (happy path)
  **Given** a user runs `wm` with no subcommand on a terminal
  **When** the command executes
  **Then** usage/help is printed and the process exits 0 — no TUI is launched

  ### Scenario 2: Explicit TUI invocation (removed surface)
  **Given** a user runs `wm tui` or `wm --tui`
  **When** the command executes
  **Then** clap rejects it as an unknown subcommand/flag and exits non-zero

  ### Scenario 3: Wizard flows preserved
  **Given** a user runs `wm init` or `wm setup` on a terminal
  **When** the command executes
  **Then** interactive prompts (dialoguer) and spinners (indicatif) still work as before

  ## Technical Notes

  - TUI call sites to eliminate in `apps/wm-cli/src/main.rs`: `:22` (`mod tui`), `:33-35` (`--tui` flag), `:107` (`Tui` variant), `:1203-1206` (auto-launch), `:1208-1214` (no-command message), `:1618-1623` (`Commands::Tui` arm).
  - The no-command path can use `Cli::command().print_help()` (or `print_long_help()`) before returning `Ok(())`.
  - `is-terminal` stays: it is used by the wizard at `main.rs:1349` and `:1468` for stdin detection, independent of the TUI.
  - Docs supersession: set `status: superseded` on @wiki/specs/tui-polish-search-scrolling-pagination-tab-cycle-unicode and `status: cancelled` on @wiki/tasks/75k8oh and @wiki/tasks/6lzncr, each with a `relates_to` link to this spec.
  - `tuistory` in `apps/wm-web/package.json` is an unrelated Angular UI-testing tool — out of scope.

  ## Open Questions

  - (None — decisions D1–D3 cover the gray areas.)
questions:
  - id: kind
    type: choice
    instructions: What kind of spec is this?
    options:
    - feature
    - system
    - doc
    - migration
    - experiment
  - id: scope
    type: choice
    instructions: How wide is the scope of this spec?
    options:
    - local
    - component
    - system
    - project-wide
  - id: status_class
    type: choice
    instructions: What lifecycle class is this spec in?
    options:
    - draft
    - reviewed
    - approved
    - superseded
  - id: needs_tasks
    type: noul
    instructions: This spec requires one or more task pages.
answers: {}
