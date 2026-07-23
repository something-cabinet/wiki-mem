---
title: Standardize dialog loading states and error reporting across all views
type: task
status: todo
priority: high
tags: [ux, web-ui, consistency]
---

From @designer review H4+H5: (1) Pages Create button has no loading/disable guard — double-click creates duplicates. (2) Pages shows error as alert AND toast simultaneously. (3) Memory delete lacks loading spinner (Pages has it). Fix: add loading to Create/Delete, pick single error channel (inline for forms, toast for background ops).