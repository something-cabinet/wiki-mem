---
id: gey6im
title: Implement code intelligence MCP tools
status: done
priority: high
labels:
  - feature
  - code-intelligence
  - knowns-parity
createdAt: '2026-07-08T11:16:23.283Z'
updatedAt: '2026-07-09T07:54:58.180Z'
timeSpent: 0
---
# Implement code intelligence MCP tools

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
WM has no code intelligence. Knowns provides knowns_code with AST-based search, symbol lookup, dependency graphs. WM needs equivalent tools for code search across the codebase. This is the largest feature gap.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Code search tool exists (search by symbol name, text pattern)
- [x] #2 Symbol lookup tool exists (find definitions, references)
- [x] #3 Dependency graph tool exists (file imports, module deps)
<!-- AC:END -->

