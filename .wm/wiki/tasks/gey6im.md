---
title: Implement code intelligence MCP tools
type: task
status: done
tags: [feature, code-intelligence, knowns-parity]
priority: high
id: gey6im
---

# Implement code intelligence MCP tools

> *Imported from Knowns task `gey6im`*

# Implement code intelligence MCP tools

## Description


WM has no code intelligence. Knowns provides knowns_code with AST-based search, symbol lookup, dependency graphs. WM needs equivalent tools for code search across the codebase. This is the largest feature gap.


## Acceptance Criteria

- [x] #1 Code search tool exists (search by symbol name, text pattern)
- [x] #2 Symbol lookup tool exists (find definitions, references)
- [x] #3 Dependency graph tool exists (file imports, module deps)
