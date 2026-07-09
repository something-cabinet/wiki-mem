---
id: ulutfi
title: Add input JSON schemas to all MCP tools
status: done
priority: high
labels:
  - feature
  - mcp
  - schemas
  - knowns-parity
createdAt: '2026-07-08T11:16:27.230Z'
updatedAt: '2026-07-09T07:54:56.333Z'
timeSpent: 0
---
# Add input JSON schemas to all MCP tools

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
WM's tools/list returns empty inputSchema for all tools. AI agents can't discover what parameters each tool accepts. Add proper JSON schemas (property names, types, descriptions, required fields) to all tool registrations so agents can self-discover parameters.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 tools/list returns inputSchema with typed properties for each tool
- [x] #2 Required fields marked as required in schema
- [x] #3 Descriptions explain each parameter's purpose
<!-- AC:END -->

