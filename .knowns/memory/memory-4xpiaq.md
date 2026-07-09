---
id: 4xpiaq
title: MCP response enrichment pattern — match Knowns depth
layer: project
category: pattern
tags:
  - mcp
  - responses
  - knowns-parity
  - format
createdAt: '2026-07-09T07:54:42.220Z'
updatedAt: '2026-07-09T07:54:42.220Z'
---

WM tool responses should match Knowns response depth for AI agent compatibility. Key enrichments: wm_doc.list returns tags/description/timestamps per doc, wm_task.board returns full task detail (ACs, timestamps, priority, timeSpent) per task, wm_memory.list returns content/camelCase dates. Response format uses camelCase JSON (createdAt, updatedAt) matching Knowns convention.
