---
---

---
id: wiki:patterns:dev-workflow-target-binary-mcp
title: "Pattern: Dev Workflow — Point MCP Config to Target Binary"
type: pattern
status: reviewed
tags: [pattern, development, workflow, mcp, debugging]
---
id: wiki:patterns:dev-workflow-target-binary-mcp

## Problem

During development, the installed `wm` binary (from npm/cargo install, on PATH) may be stale — it was built from an older commit. Running `cargo build` and expecting the MCP server to use the new code doesn't work because the MCP host resolves `wm-cli` from PATH, which points to the installed release.

This means every code change requires: `cargo build` → restart MCP server.

## Solution

Configure the MCP config (e.g. `.mcp.json` / `opencode.json`) to point the host at the **freshly-built target binary** instead of the installed release:

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
- The path is relative to the workspace root (where the config lives)
- `--` separates `mcpmon` options from the command to manage
- Debug binary preserves debug symbols, line numbers, and logging

### Variants

| Binary | When to use |
|--------|-------------|
| `./target/debug/wm-cli.exe` | Daily development, debugging |
| `./target/release/wm-cli.exe` | Performance testing, release testing |
| `wm-cli` (PATH, npm/cargo install) | Production / installed version (default) |

### Note on the MCP config command

`wm setup opencode` (and all MCP config generation) writes the canonical `["wm-cli", "mcp"]` command **unconditionally** — no path resolution or detection. The user is assumed to have `wm-cli` on PATH (via npm or cargo install). For development, manually override the command path in the config to point to your debug binary following the table above.

## When to Use

- Any Rust project with an MCP server binary
- When debugging MCP tool behavior
- When you need `RUST_LOG=debug` output from the MCP server

## When Not to Use

- Production/customer-facing MCP servers (use installed version)
- CI/CD pipelines (use release binary)
- When the MCP binary path is fragile (e.g., symlinks that might be stale)

## Related

- @wiki/specs:local-knowledge-engine-rust — Development Workflow
- @wiki/decisions:init-setup-separation — init/setup separation (path resolution removed 2026-07-31)
- @wiki/specs:remove-self-install-flow — self-install removed; MCP config always "wm-cli"