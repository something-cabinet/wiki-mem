---
title: "Decision: Codex Uses TOML Config Format"
type: decision
tags: [codex, config, toml, platform]
status: reviewed
confidence: high
decision:
  context: |
    Codex uses `.codex/config.toml` with `[mcp_servers]` sections for MCP server configuration. WM was incorrectly generating `.mcp.json` (JSON) for Codex, treating it the same as Claude Code. Since both were in a combined match arm, Codex users would get an unreadable config file.
  options:
    - "Keep combined arm with JSON format (breaks Codex)"
    - "Split arms, Codex gets TOML, Claude gets JSON"
  rationale: |
    Each platform's config format must match what the platform expects. Codex does not read `.mcp.json`. The combined arm was a shortcut that caused silent failure.
  outcome: |
    Codex now gets `.codex/config.toml` with TOML `[mcp_servers.wm]` format. Claude keeps `.mcp.json` with JSON `mcpServers`. Combined arm split into separate handlers.
relates_to:
  - {type: references, target: wiki:tasks:task-wkm5xh-research-platform-configskill-dirs-from-knowns-source-validate-wm-parity}
---

## Context

The `"claude" | "codex"` match arm treated both platforms identically, generating `.mcp.json` with JSON `mcpServers`. Codex expects `.codex/config.toml` with TOML `[mcp_servers]` format.

## Chosen approach

Split the arm. Codex generates TOML, Claude generates JSON.

## Outcome

Both platforms get correct config. TOML writer is a simple format!() call — no dependency needed.

## Source

@wiki/tasks/wkm5xh
