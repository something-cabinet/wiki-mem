---
title: MCP tool input schema pattern — register_with_schema()
type: memory
tags: [mcp, schemas, tool-discovery, ai]
created_at: "2026-07-09T07:54:40.460Z"
updated_at: "2026-07-09T07:54:40.460Z"
---

ToolRegistry should expose input JSON schemas per tool via tools/list. Added register_with_schema(name, desc, schema_json, handler) to ToolRegistry. Each tool declares its parameters with types, descriptions, defaults, and required fields. AI agents use these schemas to self-discover what arguments a tool accepts — no trial-and-error needed. Maps to MCP protocol's inputSchema field.