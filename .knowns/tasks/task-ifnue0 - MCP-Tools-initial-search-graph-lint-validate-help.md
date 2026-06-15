---
id: ifnue0
title: MCP Tools (initial, search, graph, lint, validate, help, audit, permissions)
status: done
priority: high
labels:
  - from-spec
  - go-mode
  - mcp-tools
createdAt: '2026-06-15T11:31:30.167Z'
updatedAt: '2026-06-15T14:09:55.150Z'
timeSpent: 0
spec: specs/local-knowledge-engine-rust
fulfills:
  - AC-12
  - AC-16
  - AC-17
---
# MCP Tools (initial, search, graph, lint, validate, help, audit, permissions)

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
initial tool (project state + conventions), search.query/retrieve (BM25 + context assembly + mode parameter), graph.neighbors (topic-aware sorting), lint.check (orphan pages, broken refs, missing ACs), validate (per-type frontmatter completeness), source.list/status, help tool (tool documentation registry), audit logging (bounded channel), permission guard middleware, rotating file logger
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
<!-- AC:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Registered 12 MCP tool handlers: wm_initial (project state), wm_search.query (BM25 with mode param, type filter), wm_page.get/create/list, wm_source.add/process/complete/list/verify, wm_graph.neighbors (topic-aware BFS with edge weights), wm_lint.check (orphan detection), wm_validate.check (graph health). All tools wired to core modules via mcp/tools.rs. Graph loaded from wiki dir on serve.
<!-- SECTION:NOTES:END -->

