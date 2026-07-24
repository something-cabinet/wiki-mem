---
id: wiki:patterns:dev-workflow-target-binary-mcp
title: "Pattern: Dev Workflow — Point MCP Config to Target Binary"
type: pattern
status: reviewed
tags: [pattern, development, workflow, mcp, debugging]
---
id: wiki:patterns:dev-workflow-target-binary-mcp

## Problem

During development, the installed `wm` binary at `~/.wm/bin/wm` may be stale — it was built from an older commit. Running `cargo build` and expecting the MCP server to use the new code doesn't work because `mcpmon` resolves `wm` from PATH, which points to the installed release.

This means every code change requires: `cargo build` → `wm-cli install` → restart MCP server.

## Solution

Configure Reasonix's `.mcp.json` to point `mcpmon` at the **freshly-built target binary** instead of the installed release:

```json
{
  "mcpServers": {
    "wm": {
      "args": ["--", "./target/debug/wm-cli.exe", "mcp"],
      "command": "mcpmon"
    }
  }
}
```

Now the workflow is:
1. `cargo build` — compile changes
2. Restart MCP server in the IDE (or let `mcpmon` hot-reload)
3. Test immediately — no install step

### Why this works

- `mcpmon` manages the process lifecycle — restarts on crash, hot-reloads
- The path is relative to the workspace root (where `.mcp.json` lives)
- `--` separates `mcpmon` options from the command to manage
- Debug binary preserves debug symbols, line numbers, and logging

### Variants

| Binary | When to use |
|--------|-------------|
| `./target/debug/wm-cli.exe` | Daily development, debugging |
| `./target/release/wm-cli.exe` | Performance testing, release testing |
| `~/.wm/bin/wm` | Production / installed version (default) |

## When to Use

- Any Rust project with an MCP server binary
- When debugging MCP tool behavior
- When you need `RUST_LOG=debug` output from the MCP server

## When Not to Use

- Production/customer-facing MCP servers (use installed version)
- CI/CD pipelines (use release binary)
- When the MCP binary path is fragile (e.g., symlinks that might be stale)

## Related

- @doc/specs/single-http-server — Development Workflow section
- @doc/decisions/wm-server-overrides-tauri-primary — Architecture decision
