---
title: Knowns platform config paths reference
type: memory
tags: [knowns, platform, config, reference]
created_at: "2026-07-06T18:50:57.752Z"
updated_at: "2026-07-06T18:50:57.752Z"
---

Knowns supports 7+ platforms with distinct config paths:
- Claude Code: `.mcp.json` (JSON, mcpServers), global at Claude Desktop config
- Codex: `.codex/config.toml` (TOML, [mcp_servers])
- OpenCode: `opencode.json` (JSON, mcp), global at ~/.config/opencode/
- Kiro: `.kiro/settings/mcp.json` (JSON, mcpServers), global same path
- Cursor: `.cursor/mcp.json` (JSON, mcpServers)
- Antigravity: `~/.gemini/antigravity/mcp_config.json` (always global)
- Skills: `.agents/skills/` (shared), `.claude/skills/`, `.kiro/skills/` (per-platform)