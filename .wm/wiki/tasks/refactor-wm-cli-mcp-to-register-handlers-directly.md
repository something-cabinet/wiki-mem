---
title: Refactor wm-cli mcp to register handlers directly
type: task
status: todo
priority: high
tags: [from-spec, mcp]
spec: specs/mcp-tool-registry-unification
---

Replace HTTP proxy handlers with direct register_all_tools call. Remove HTTP server startup from MCP mode. Remove ureq dependency. wm-cli mcp becomes MCP-only — no background HTTP server.