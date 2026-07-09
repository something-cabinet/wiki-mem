---
id: 11s8gj
title: 'Failure: Tool rename caused double wm_wm_ prefix'
layer: project
category: failure
tags:
  - mcp
  - naming
  - prefix
createdAt: '2026-07-09T08:01:45.676Z'
updatedAt: '2026-07-09T08:01:45.676Z'
---

Removing the wm_ prefix from tool names to avoid double prefix (wm_wm_doc_list) broke collision safety. Knowns' critical pattern says prefix MCP tools to avoid collisions. Reverted. The double prefix is cosmetic in OpenCode only — other clients (Claude Code, Kiro) don't add server prefix, so the wm_ prefix is needed there.
