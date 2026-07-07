---
id: 13wnqe
title: Codex TOML vs JSON config format
layer: project
category: failure
tags:
  - codex
  - config
  - toml
createdAt: '2026-07-06T18:50:55.788Z'
updatedAt: '2026-07-06T18:50:55.788Z'
---

Codex uses `.codex/config.toml` with `[mcp_servers.wm]` TOML sections, NOT `.mcp.json` with JSON. The claude/codex combined arm was wrong because they diverge in format and path. Always research each platform's documented config format independently.
