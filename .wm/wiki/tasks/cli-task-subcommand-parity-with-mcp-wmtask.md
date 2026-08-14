---
title: CLI task subcommand parity with MCP wm_task
type: task
id: wiki:tasks:cli-task-subcommand-parity-with-mcp-wmtask
status: todo
priority: medium
tags: [parity, cli, task]
acceptance_criteria:
  - text: "CLI task subcommand exposes the same actions as MCP wm_task (list, create, get, update, delete, check_ac, uncheck_ac, subtask, board)"
  - text: "At minimum task get and task list are available via CLI for debugging task state"
---

CLI/MCP parity gap found while debugging task IDs: wm-cli task exposes only the board subcommand while MCP wm_task has 9 actions (board, list, create, get, update, delete, check_ac, uncheck_ac, subtask). Could not inspect or fix task state via CLI during the wm-doc-fix wave; had to rely on MCP tools plus direct file inspection. Principle: CLI should be 1:1 with MCP.