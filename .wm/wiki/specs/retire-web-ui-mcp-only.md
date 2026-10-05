---
title: Retire Web UI — 0.6 Ships MCP + CLI Only
type: spec
id: "wiki:specs:retire-web-ui-mcp-only"
status: draft
tags: ['0.6', retirement, mcp-only, wm-web, wm-server, cleanup]
---

## Overview

For release **0.6**, WM ships as **MCP + CLI only**. The browser surface is retired: the Angular SPA (`wm-web`), the Axum HTTP daemon (`wm-server`), the `wm web` command, and any browser-only artifacts. `wm-cli` (in-process dispatch) and the MCP server (`wm-cli mcp`) remain.

## Locked Decisions

- **D1**: Delete `apps/wm-web` (Angular SPA).
- **D2**: Delete `apps/wm-server` (Axum daemon — web-UI only per `@doc/core:ARCHITECTURE`).
- **D3**: Remove the `wm web` CLI command (and `wm serve`/daemon startup) from `wm-cli`.
- **D4**: MCP stays **in-process** (`wm-cli mcp`, rmcp stdio) — no daemon, no HTTP proxy, no tokens.
- **D5**: Remove browser-only WASM crates and embedded web assets if nothing else consumes them (e.g. `fjadra-wasm`, `graph-algo-wasm`, `bm25-rerank-wasm`, `md-parse-wasm`, web-token/`server.json` handling, SPA assets).
- **D6**: CLI commands continue (search/page/task/graph/index/validate/etc.).

## Requirements

### Functional Requirements
- **FR-1**: Remove `wm-web` and `wm-server` from the workspace (members, deps, lockfile).
- **FR-2**: Remove `wm web` / `wm serve` commands and any daemon lifecycle, port, singleton-guard, web-token/mcp-token, CORS code.
- **FR-3**: Remove web-only test suites (e2e HTTP-against-daemon, SPA tests) and CI references.
- **FR-4**: Remove browser-only WASM crates/assets that no remaining target uses.
- **FR-5**: Keep MCP + CLI working in-process; `wm-cli mcp` serves stdio with no daemon.
- **FR-6**: Update docs (`README`, `core/ARCHITECTURE`, `core/CONVENTIONS`) to drop the web/daemon deployment mode and any web-UI claims.

### Non-Functional Requirements
- **NFR-1**: Workspace builds with zero warnings after removal.
- **NFR-2**: No dead code or dangling references to the removed crates/commands.
- **NFR-3**: MCP + CLI behaviour unchanged for existing users.

## Acceptance Criteria
- [ ] AC-1: `apps/wm-web` and `apps/wm-server` no longer exist; not workspace members.
- [ ] AC-2: `wm web` / `wm serve` are gone; CLI help no longer lists them.
- [ ] AC-3: No references to the removed crates/commands remain (grep-clean), except historical wiki pages.
- [ ] AC-4: `cargo check --workspace` + `cargo clippy` clean; tests pass.
- [ ] AC-5: `wm-cli mcp` and core CLI commands still work.
- [ ] AC-6: Docs updated; no stale web/daemon instructions.

## Scenarios
### Scenario 1: MCP-only user
**Given** a fresh checkout
**When** the user runs `wm-cli mcp` (or CLI commands)
**Then** everything works with no daemon/web build.

### Scenario 2: No dangling references
**Given** the removal is done
**When** grepping the workspace
**Then** no source references `wm-web`, `wm-server`, `wm web`, or browser-only assets.

## Technical Notes
- `@doc/core:ARCHITECTURE` already defines `wm-server` as opt-in web-UI-only; verify nothing else depends on it before deletion.
- WASM crates are "pure compute for the browser"; retire them with the UI unless a CLI/other consumer uses them.
- Order: retire the web stack **before** the comment sweep (`@doc/specs/ban-all-comments`), so we don't clean code slated for deletion.

## Open Questions
- [ ] Retire the WASM crates too, or keep them for future reuse?
- [ ] Remove the HTTP e2e suites entirely, or convert to CLI/MCP integration tests?
