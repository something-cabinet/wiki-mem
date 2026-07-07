---
id: 8wqqm8
title: Unify CLI and MCP task board implementations
status: done
priority: low
labels:
  - from-review
  - board
  - cli
  - mcp
createdAt: '2026-07-07T08:51:04.851Z'
updatedAt: '2026-07-07T09:28:40.376Z'
timeSpent: 0
spec: specs/unify-cli-and-mcp-task-board
---
# Unify CLI and MCP task board implementations

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
P3 from architect review. Three implementations of task board exist:
1. CLI TaskAction::Board (wm-cli/src/main.rs:1816-1865) — iterates graph directly
2. MCP wm_task.board (wm-core/src/mcp/tools.rs:1496-1535) — uses ArcSwap snapshot

Extract into shared wm_core::task::board() function that both call. Low urgency since logic is trivial (iterate graph, bin by status) but reduces drift risk.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
<!-- AC:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Unified CLI/MCP task board:
- Created wm_core::task module with shared task_board() function
- TaskBoard and TaskBoardItem structs with serde support
- Both CLI and MCP call the shared function
<!-- SECTION:NOTES:END -->

