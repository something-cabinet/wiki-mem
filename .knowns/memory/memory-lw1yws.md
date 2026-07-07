---
id: lw1yws
title: Knowns platform config paths reference
layer: project
category: pattern
tags:
  - knowns
  - platform
  - config
  - reference
createdAt: '2026-07-06T18:50:57.752Z'
updatedAt: '2026-07-06T18:50:57.752Z'
---

Knowns supports 7+ platforms with distinct config paths:
- Claude Code: `.mcp.json` (JSON, mcpServers), global at Claude Desktop config
- Codex: `.codex/config.toml` (TOML, [mcp_servers])
- OpenCode: `opencode.json` (JSON, mcp), global at ~/.config/opencode/
- Kiro: `.kiro/settings/mcp.json` (JSON, mcpServers), global same path
- Cursor: `.cursor/mcp.json` (JSON, mcpServers)
- Antigravity: `~/.gemini/antigravity/mcp_config.json` (always global)
- Skills: `.agents/skills/` (shared), `.claude/skills/`, `.kiro/skills/` (per-platform)
