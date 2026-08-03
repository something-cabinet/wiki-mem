---
title: Remove Self-Install Flow — Binary Deployment via npm/cargo only
type: spec
tags:
- spec
- install
- deploy
- npm
- cleanup
status: approved
relates_to:
  - {type: implements, target: wiki:tasks:remove-self-install-flow-wm-upgrade-install-module---full-flag}
---

## Overview

WM's self-install flow (`wm upgrade` / `wm init --full` → copies the running binary to `~/.wm/bin/` and registers PATH) was designed in @wiki/specs/wm-self-install to mirror Knowns' `~\.knowns\bin\` pattern. It predates the current distribution channel: **cargo-npm** (`@something-cabinet/wm-cli` metapackage + per-platform native binaries + `wm-server`, per @wiki/memory/wm-cli-web-must-bundle-wm-server-in-npm-package).

npm already provides a stable, per-user, PATH-registered install (npm bin symlink into the node bin dir), and `cargo install` covers Rust-toolchain users. The self-install is redundant on all platforms — including win32-x64, which npm covers. On macOS it is additionally broken: `ensure_on_path()` writes `~/.profile`, which zsh never sources, leaving a stale ~170MB binary in `~/.wm/bin/` that nothing references.

This spec removes the self-install flow entirely. Distribution becomes npm-only (with `cargo install` as the Rust-native alternative), and generated MCP configs **always** reference `wm-cli` on PATH — the user is assumed to have installed it.

## Locked Decisions

- **D1 — Remove entirely, all platforms**: delete `wm upgrade`, the install module (`apps/wm-core/src/install/`), and the install step inside `wm init`. npm and cargo channels cover all platforms.
- **D2 — Remove `--full` flag**: `wm init --full` loses its purpose once the install step is gone; the flag is removed, leaving plain `wm init`.
- **D3 — MCP config always uses `wm-cli`**: `wm setup opencode` and all MCP config generation write `"command": "wm-cli"` unconditionally. No `~/.wm/bin` existence check, no PATH lookup, no `current_exe()` fallback. The user is assumed to have `wm-cli` on PATH (via npm or cargo install).
- **D4 — Leave existing installs**: `~/.wm/bin/` folders already present are untouched (no surprise deletions). Manual removal (`rm -rf ~/.wm/bin`) is documented in this spec and its follow-up task.

## Requirements

### Functional Requirements

- **FR-1**: Remove the `wm upgrade` CLI command (clap variant + dispatch arm in `apps/wm-cli/src/main.rs:1458`).
- **FR-2**: Remove the install module `apps/wm-core/src/install/mod.rs` (`install_dir`, `is_installed`, `install_binary`, `ensure_on_path`, `check_status`, `is_on_path`, `exe_name`) and its module wiring in `wm-core`.
- **FR-3**: Remove the install + PATH-registration step from `wm init` (`apps/wm-cli/src/main.rs:1095-1100`) and remove the `--full` flag.
- **FR-4**: Simplify `resolve_mcp_binary()` (`apps/wm-cli/src/main.rs:785`) to always return `"wm-cli"`. Remove the `is_installed()` check and the `current_exe()` fallback. If the function becomes trivial, remove it and inline the constant.
- **FR-5**: Update docs to reflect the removed flow: `WIKI-MEM.md` Quick Reference (remove `wm upgrade`, `wm init --full` lines), mark @wiki/specs/wm-self-install and @wiki/decisions/wm-self-upgrade superseded, and update @wiki/patterns/wm-init-full and any memory entries referencing the install flow.
- **FR-6**: Update or remove tests referencing the install flow (wm-cli unit tests, e2e tests that invoke `install_binary` or `wm upgrade`, or that assert absolute-path MCP output).
- **FR-7**: Document manual cleanup of legacy `~/.wm/bin/` folders (in this spec and the implementation task notes).

### Non-Functional Requirements

- **NFR-1**: No dead code left behind — no `#[allow(dead_code)]`; the install module and all call sites are deleted, not commented out.
- **NFR-2**: Zero compiler warnings — `cargo build` and `cargo clippy` clean.
- **NFR-3**: Generated MCP configs are identical on every machine: `"command": "wm-cli"`.
- **NFR-4**: If `wm-cli` is not on PATH, MCP startup fails loudly with the standard "command not found" — no hidden install attempt, no silent absolute-path fallback.

## Acceptance Criteria

- [ ] AC-1: `wm upgrade` is not a recognized command (`wm --help` omits it; invoking it errors).
- [ ] AC-2: `wm init --full` is rejected as an unknown flag (or absent from `wm init --help`).
- [ ] AC-3: `wm init` never creates or modifies `~/.wm/bin/`.
- [ ] AC-4: `wm setup opencode` writes `"command": "wm-cli"` unconditionally — regardless of PATH contents, `~/.wm/bin` presence, or how the binary was launched.
- [ ] AC-5: No `wm_core::install` reference remains in the codebase.
- [ ] AC-6: `cargo build` and `cargo test` pass with zero warnings.
- [ ] AC-7: `WIKI-MEM.md` no longer documents `wm upgrade` or `wm init --full`.
- [ ] AC-8: @wiki/specs/wm-self-install, @wiki/decisions/wm-self-upgrade, and @wiki/patterns/wm-init-full are marked superseded and link to this spec.

## Scenarios

### Scenario 1: npm user generates MCP config (happy path)
**Given** `npm i -g @something-cabinet/wm-cli` was run (so `wm-cli` is on PATH) and no `~/.wm/bin` exists
**When** `wm setup opencode` runs
**Then** the MCP config contains `"command": "wm-cli"`, and no `~/.wm/bin` directory is created

### Scenario 2: Dev build from target/debug
**Given** the developer runs `target/debug/wm-cli` and `wm-cli` is not on PATH
**When** `wm setup opencode` runs
**Then** the MCP config still contains `"command": "wm-cli"`; the developer is responsible for having it on PATH (npm install or symlink) before the MCP host starts

### Scenario 3: Legacy install present
**Given** `~/.wm/bin/wm-cli` exists from an earlier self-install
**When** any `wm` command runs
**Then** behavior is identical to a clean machine; the legacy folder is left untouched (manual removal documented)

## Technical Notes

- `wm_core::install` call sites to eliminate: `apps/wm-cli/src/main.rs:786` (`resolve_mcp_binary`), `:1096` (`wm init --full`), `:1458-1465` (`wm upgrade`). Check for module declaration in `wm-core` lib and any tests.
- `resolve_mcp_binary()` becomes a constant `"wm-cli"`. If unused elsewhere, delete the function and inline the string at the `patch_mcp_command` call site.
- Docs supersession: set `status: superseded` on @wiki/specs/wm-self-install and @wiki/decisions/wm-self-upgrade with a `relates_to` link to this spec.

## Open Questions

- (None — decisions D1–D4 cover the gray areas.)