---
title: Unify CLI and MCP task board implementations
type: task
status: done
tags: [from-review, board, cli, mcp]
priority: low
id: 8wqqm8
spec: specs/unify-cli-and-mcp-task-board
relates_to:
  - {type: implements, target: wiki:specs:unify-cli-and-mcp-task-board}
acceptance_criteria:
  - text: "A shared wm_core::task::board() function exists and is called by both CLI TaskAction::Board and MCP wm_task.board"
  - text: "TaskBoard and TaskBoardItem structs with serde support produce identical board output from CLI and MCP"
---

# Unify CLI and MCP task board implementations

> **Spec:** `specs/unify-cli-and-mcp-task-board`

> *Imported from Knowns task `8wqqm8`*

# Unify CLI and MCP task board implementations

## Description


P3 from architect review. Three implementations of task board exist:
1. CLI TaskAction::Board (wm-cli/src/main.rs:1816-1865) — iterates graph directly
2. MCP wm_task.board (wm-core/src/mcp/tools.rs:1496-1535) — uses ArcSwap snapshot

Extract into shared wm_core::task::board() function that both call. Low urgency since logic is trivial (iterate graph, bin by status) but reduces drift risk.


## Acceptance Criteria



## Implementation Notes


Unified CLI/MCP task board:
- Created wm_core::task module with shared task_board() function
- TaskBoard and TaskBoardItem structs with serde support
- Both CLI and MCP call the shared function
