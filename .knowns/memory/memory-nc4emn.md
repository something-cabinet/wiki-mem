---
id: nc4emn
title: Platform-aware MCP config generation pattern
layer: project
category: pattern
tags:
  - mcp
  - platform
  - config
  - knowns
createdAt: '2026-07-06T18:50:54.612Z'
updatedAt: '2026-07-06T18:50:54.612Z'
---

When building a CLI that integrates with AI coding assistants, each platform has its own MCP config path, format, and key structure. Map all platforms first, then separate init (agent shims) from setup (MCP config). Per-platform skills directories (`.claude/skills/`, `.kiro/skills/`) match Knowns' pattern. Codex uses TOML, not JSON. --global flag routes to user-level config paths.
