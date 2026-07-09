---
title: Add input JSON schemas to all MCP tools
type: task
status: done
tags: [feature, mcp, schemas, knowns-parity]
priority: high
knowns_id: ulutfi
---

# Add input JSON schemas to all MCP tools

> *Imported from Knowns task `ulutfi`*

# Add input JSON schemas to all MCP tools

## Description


WM's tools/list returns empty inputSchema for all tools. AI agents can't discover what parameters each tool accepts. Add proper JSON schemas (property names, types, descriptions, required fields) to all tool registrations so agents can self-discover parameters.


## Acceptance Criteria

- [x] #1 tools/list returns inputSchema with typed properties for each tool
- [x] #2 Required fields marked as required in schema
- [x] #3 Descriptions explain each parameter's purpose
