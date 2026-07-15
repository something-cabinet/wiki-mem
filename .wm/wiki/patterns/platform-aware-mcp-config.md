---
title: "Pattern: Platform-Aware MCP Config Generation"
type: pattern
tags: [mcp, platform, config, setup]
status: reviewed
confidence: high
relates_to:
  - {type: references, target: wiki:patterns:mcp-response-format}
  - {type: implemented-by, target: wiki:decisions:init-setup-separation}
---

## When to use

When building a CLI tool that integrates with multiple AI coding assistants (Claude Code, Codex, OpenCode, Kiro, Cursor, etc.). Each platform has its own config file location, format, and key structure for MCP server registration.

## How it works

1. **Identify all target platforms** — research each platform's MCP config convention
2. **Map each platform to three dimensions**:
   - Config file path (project-local vs global)
   - Config format (JSON, TOML)
   - Key structure (mcpServers, mcp, [mcp_servers])
3. **Separate concerns**: `init` generates agent instruction files, `setup` generates MCP configs
4. **Support --global flag** for user-level config (e.g., `~/.config/opencode/`, `~/.kiro/`, `%APPDATA%/Claude/`)
5. **Merge with existing config** — don't overwrite other MCP server entries

## Platform mapping reference

| Platform | Project config | Global config | Format | Key |
|----------|---------------|---------------|--------|-----|
| Claude Code | `.mcp.json` | `%APPDATA%/Claude/claude_desktop_config.json` | JSON | mcpServers |
| Codex | `.codex/config.toml` | `~/.codex/config.toml` | TOML | [mcp_servers] |
| OpenCode | `opencode.json` | `~/.config/opencode/opencode.json` | JSON | mcp |
| Kiro | `.kiro/settings/mcp.json` | `~/.kiro/settings/mcp.json` | JSON | mcpServers |
| Cursor | `.cursor/mcp.json` | `~/.cursor/mcp.json` | JSON | mcpServers |
| Antigravity | N/A (global only) | `~/.gemini/antigravity/mcp_config.json` | JSON | mcpServers |

## Source

@wiki/tasks/omuamh @wiki/tasks/wkm5xh
