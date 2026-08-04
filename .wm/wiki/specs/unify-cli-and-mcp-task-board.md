---
title: Unify CLI and MCP Task Board
type: spec
tags:
  - spec
  - approved
  - board
  - cli
  - mcp
---
id: wiki:specs:unify-cli-and-mcp-task-board

## Overview

Three implementations of task board exist:
1. CLI TaskAction::Board (wm-cli/src/main.rs:1816-1865) — iterates graph directly
2. MCP wm_task.board (wm-core/src/mcp/tools.rs:1496-1535) — uses ArcSwap snapshot

Extract into shared `wm_core::task::board()` function that both call. Logic is trivial (iterate graph, bin by status) so urgency is low, but reduces drift risk.

## Locked Decisions

- D1: Shared function lives in `wm_core::task::board()` returning `Vec<TaskBoardColumn>`
- D2: Both CLI and MCP call the same function — no inline alternatives
- D3: CLI and MCP output formats remain independent (CLI formats as text/JSON, MCP as JSON-RPC result)

## Requirements

### Functional Requirements

- FR-1: Task board logic must live in exactly one place
- FR-2: CLI `wm task board` must produce same data as MCP `wm_task.board`
- FR-3: Both implementations must use the same graph iteration and status binning

### Non-Functional Requirements

- NFR-1: `cargo build` and `cargo test` pass without new warnings
- NFR-2: Output format/display code stays in CLI/MCP respectively

## Acceptance Criteria

- [ ] AC-1: `wm_core::task::board()` function exists with signature returning structured board data
- [ ] AC-2: CLI main.rs TaskAction::Board calls `wm_core::task::board()`
- [ ] AC-3: MCP tools.rs wm_task.board calls `wm_core::task::board()`
- [ ] AC-4: No inline board logic remains in CLI or MCP code
- [ ] AC-5: All existing board tests pass

## Scenarios

### Scenario 1: Consistent board across interfaces
**Given** a project with tasks in various statuses
**When** user runs `wm task board` and calls `wm_task.board` via MCP
**Then** both return identical task counts per status column
