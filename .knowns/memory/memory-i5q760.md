---
id: i5q760
title: 'Decision: Migrate to rmcp (official MCP Rust SDK)'
layer: project
category: decision
tags:
  - mcp
  - rmcp
  - rust
  - good-call
createdAt: '2026-07-09T17:16:09.212Z'
updatedAt: '2026-07-09T17:16:09.212Z'
---

Replaced hand-rolled MCP transport with rmcp v2.2.0. 302 lines → ~100 lines. Fixed content[] wrapper compliance. All 148 tests pass. Keep ToolRegistry as-is, wrap with ServerHandler. Full reference: @doc/learnings/learning-rmcp-mcp-sdk-migration
