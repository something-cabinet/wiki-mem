---
title: T3: Remove wm-cli serve, update wm-cli web
type: task
status: done
priority: medium
tags: [from-spec, spec/mcp-direct-handlers]
---

## Description

Remove the `wm-cli serve` command (no longer needed — MCP is started by `mcpmon`, HTTP is started by `wm-cli web`). Ensure `wm-cli web` starts the HTTP server only.

## Acceptance Criteria

- [ ] AC-5: `wm-cli serve` no longer exists as a command
- [ ] AC-6: `wm-cli web` starts the HTTP server only; no MCP spawning

## Fulfills

- FR-4: `wm-cli serve` command removed
- FR-5: `wm-cli web` starts HTTP server only

## Files

- `apps/wm-cli/src/main.rs` — command definitions
