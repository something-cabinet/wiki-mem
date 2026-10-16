---
title: Shim template command parity guard
type: spec
id: "wiki:specs:shim-template-command-parity-guard"
status: draft
tags: [testing, agent-guidance, ci, embedded-templates]
implementation_notes: 'Amended 2026-10-15: FR-2 must cover inline backtick spans, not only fenced bash blocks — proven by mutation that an inline `wm-cli bogus` passed the guard, and the original defect included an inline invocation. Add an AC for the inline case. Implementation was reverted to avoid sharing a non-compiling build target; re-add after the wm-cli main.rs rewrite lands. Details and required verification steps: @wiki/tasks/shim-template-command-parity-guard'
---

schema_version: 1
state: |-

  ## Overview

  A guard test that fails when an embedded agent-handbook template names a CLI command that does not exist.

  Motivation: @wiki/specs/remove-self-install-flow removed `wm upgrade` and `wm init --full` from the CLI and required, in FR-5 / AC-7 / AC-8, that the handbooks stop documenting them. The code half landed; the doc half did not, and nothing failed. Six templates still ship `wm-cli serve`, `wm upgrade` and `wm init --full`, and `wm-cli init` copies them into every user's project. This spec adds the check that was missing.

  Scope is the guard only. The template content fix is @wiki/tasks/agent-handbooks-reference-nonexistent-cli-commands

  ## Locked Decisions

  - **D1 — The test lives in `apps/wm-cli`.** It is the only crate that can see both the embedded templates (via wm-core's `EmbeddedFiles`) and the clap command definition. No new crate, no dependency change.
  - **D2 — Enumerate commands from clap, never from a hand-written list.** Use clap's `CommandFactory` to walk the real subcommand tree, so the test cannot drift from the CLI it guards.
  - **D3 — Assert the binary name too.** A shim may not invoke the bare `wm` command; the shipped binary is `wm-cli`.
  - **D4 — Also assert the checked-in copies match the templates.** The root handbooks are generated artifacts; drift between them and the templates is the same class of defect.

  ## Requirements

  ### Functional Requirements

  - **FR-1** — The test loads every template under `apps/wm-core/src/embed_files/shims/` from the embedded file set (not from disk), so it tests what actually ships.
  - **FR-2** — It extracts command invocations from fenced `bash` blocks, taking the first two tokens (command + subcommand) and ignoring comments, flags, and argument placeholders.
  - **FR-3** — Every extracted subcommand must resolve in the clap subcommand tree, walking one level deep so nested forms such as `search query` and `lint check` are checked, not just `search`.
  - **FR-4** — Any invocation of the bare `wm` command fails the test.
  - **FR-5** — Failure output names the template file and the offending line, so the fix is obvious.
  - **FR-6** — A separate assertion checks that each checked-in root handbook is byte-identical to its template counterpart.
  - **FR-7** — The test runs in the default test suite with no ONNX model, no network, and no built release binary.

  ### Non-Functional Requirements

  - **NFR-1** — No new dependencies; the test uses what the crates already depend on.
  - **NFR-2** — No `#[allow(...)]` and no compiler warnings.
  - **NFR-3** — The test is deterministic and does not depend on the current working directory.

  ## Acceptance Criteria

  - [ ] AC-1: With the current (broken) templates restored, the test fails and its output names the template file and the offending command.
  - [ ] AC-2: With the corrected templates, the test passes.
  - [ ] AC-3: Injecting `wm-cli serve` into any template makes the test fail.
  - [ ] AC-4: Injecting a bare `wm upgrade` line into any template makes the test fail.
  - [ ] AC-5: Editing a root handbook so it no longer matches its template makes the template-parity assertion fail.
  - [ ] AC-6: Deleting a subcommand from the clap definition while a template still documents it makes the test fail.
  - [ ] AC-7: `cargo test` passes with zero warnings.

  ## Scenarios

  ### Scenario 1: Regression is caught (the missing guard)
  **Given** a future change adds a command line to a shim template without adding the subcommand to the CLI
  **When** the test suite runs
  **Then** it fails, naming the template and the command.

  ### Scenario 2: Removal is caught (the original defect)
  **Given** a subcommand is removed from the CLI definition while a template still documents it
  **When** the test suite runs
  **Then** it fails — this is exactly what `remove-self-install-flow` should have triggered.

  ### Scenario 3: Generated copy drifts
  **Given** a root handbook was hand-edited and no longer matches its template
  **When** the test suite runs
  **Then** the parity assertion fails, because the next `wm-cli agents` would silently overwrite the edit.

  ## Technical Notes

  - clap's `CommandFactory::command()` on the existing `Cli` struct gives the live subcommand tree; recurse into `get_subcommands()` for nested actions.
  - Extraction should tolerate the alignment padding the templates use after the command (two or more spaces before the `#` comment).
  - Templates are embedded with `RustEmbed`, so `EmbeddedFiles::iter()` sees exactly the shipped set.
  - Keep the test in the crate's test module so it can reach the private `Cli` definition without publishing it.

  ## Open Questions

- [ ] None. Add one only if the clap tree turns out not to be introspectable for a specific nested action.