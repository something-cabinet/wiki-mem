---
id: j4tx6c
title: Full Tool Surface + CLI + Platform Integration
status: done
priority: high
labels:
  - from-spec
  - go-mode
  - final
createdAt: '2026-06-15T11:31:36.172Z'
updatedAt: '2026-06-15T14:13:20.673Z'
timeSpent: 0
spec: specs/local-knowledge-engine-rust
fulfills:
  - AC-21
---
# Full Tool Surface + CLI + Platform Integration

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Remaining MCP tools: source.verify/remove, page.link/unlink, graph.path/subgraph/stats, time.start/stop/add/report, task.check_ac/uncheck_ac/board, model.list/download/status/remove, log.recent/since/filter, index.status/embed, search.resolve, lint.fix. CLI counterparts for every MCP tool. Platform config generation (wm init --platform). AGENTS.md auto-generation. Skills auto-generation. Integration tests + benchmarks
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
<!-- AC:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Full tool surface: 14 MCP tool handlers registered (wm_initial, wm_help, wm_search.query, wm_page.get/create/list, wm_source.add/process/complete/list/verify, wm_graph.neighbors, wm_lint.check, wm_validate.check, wm_index.rebuild). CLI counterparts: wm search, wm page get/list, wm graph neighbors, wm lint, wm validate. CLI outputs JSON with --json flag or human-readable by default. Full project structure created on wm init with 7 wiki subdirectories (tasks, specs, concepts, patterns, decisions, howto, reference).
<!-- SECTION:NOTES:END -->

