---
title: MCP proxy architecture — privileged /api/mcp channel + token split
type: memory
tags: [mcp, proxy, architecture, security]
status: active
---

MCP is a thin stdio→HTTP proxy to the wm-server daemon, targeting a privileged POST /api/mcp/tools/{list,call} channel with a SEPARATE mcp-token (web-token stays read-only). Dynamic tools/list from the registry — no STATIC_TOOLS array. Full: @wiki/decisions/mcp-proxy-privileged-channel-token-split