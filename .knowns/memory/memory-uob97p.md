---
id: uob97p
title: MCP tool input schema pattern — register_with_schema()
layer: project
category: pattern
tags:
  - mcp
  - schemas
  - tool-discovery
  - ai
createdAt: '2026-07-09T07:54:40.460Z'
updatedAt: '2026-07-09T07:54:40.460Z'
---

ToolRegistry should expose input JSON schemas per tool via tools/list. Added register_with_schema(name, desc, schema_json, handler) to ToolRegistry. Each tool declares its parameters with types, descriptions, defaults, and required fields. AI agents use these schemas to self-discover what arguments a tool accepts — no trial-and-error needed. Maps to MCP protocol's inputSchema field.
