---
title: MCP Tools (initial, search, graph, lint, validate, help, audit, permissions)
type: task
status: done
tags: [from-spec, go-mode, mcp-tools]
priority: high
knowns_id: ifnue0
spec: specs/local-knowledge-engine-rust
fulfills: [AC-12, AC-16, AC-17]
---

# MCP Tools (initial, search, graph, lint, validate, help, audit, permissions)

> **Spec:** `specs/local-knowledge-engine-rust`

> **Fulfills:** AC-12, AC-16, AC-17

> *Imported from Knowns task `ifnue0`*

# MCP Tools (initial, search, graph, lint, validate, help, audit, permissions)

## Description


initial tool (project state + conventions), search.query/retrieve (BM25 + context assembly + mode parameter), graph.neighbors (topic-aware sorting), lint.check (orphan pages, broken refs, missing ACs), validate (per-type frontmatter completeness), source.list/status, help tool (tool documentation registry), audit logging (bounded channel), permission guard middleware, rotating file logger


## Acceptance Criteria



## Implementation Notes


Registered 12 MCP tool handlers: wm_initial (project state), wm_search.query (BM25 with mode param, type filter), wm_page.get/create/list, wm_source.add/process/complete/list/verify, wm_graph.neighbors (topic-aware BFS with edge weights), wm_lint.check (orphan detection), wm_validate.check (graph health). All tools wired to core modules via mcp/tools.rs. Graph loaded from wiki dir on serve.
