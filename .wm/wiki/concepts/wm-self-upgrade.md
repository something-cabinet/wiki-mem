---
title: "WM Self-Upgrade — Binary Deployment + PATH Registration"
page_type: concept
tags: [pattern, deployment, self-upgrade, knowns]
---

## The Pattern

WM follows Knowns' deployment model: the running binary copies itself to a well-known location (`~\.wm\bin\wm-cli.exe`) and registers it on the user PATH. This enables:

- Platform MCP configs to reference `wm-cli` by name instead of a fragile `target/debug/` path
- `wm init --full` chains upgrade → config → project init in one command
- The opencode MCP config uses `"command": "wm-cli", "args": ["mcp"]` when installed

## Implementation

- `packages/wm-install` — `install_binary()`, `ensure_on_path()`, `check_status()`
- Windows PATH management via `REG ADD HKCU\Environment`
- Features: `wm upgrade` (standalone), `wm init --full` (chained)

## Source

@wiki/concepts/knowns-deployment (mirrors `~\.knowns\bin\knowns.exe`)
