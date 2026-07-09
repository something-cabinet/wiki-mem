---
title: Full Tool Surface + CLI + Platform Integration
type: task
status: done
tags: [from-spec, go-mode, final]
priority: high
knowns_id: j4tx6c
spec: specs/local-knowledge-engine-rust
fulfills: [AC-21]
---

# Full Tool Surface + CLI + Platform Integration

> **Spec:** `specs/local-knowledge-engine-rust`

> **Fulfills:** AC-21

> *Imported from Knowns task `j4tx6c`*

# Full Tool Surface + CLI + Platform Integration

## Description


Remaining MCP tools: source.verify/remove, page.link/unlink, graph.path/subgraph/stats, time.start/stop/add/report, task.check_ac/uncheck_ac/board, model.list/download/status/remove, log.recent/since/filter, index.status/embed, search.resolve, lint.fix. CLI counterparts for every MCP tool. Platform config generation (wm init --platform). AGENTS.md auto-generation. Skills auto-generation. Integration tests + benchmarks


## Acceptance Criteria



## Implementation Notes


Full tool surface: 14 MCP tool handlers registered (wm_initial, wm_help, wm_search.query, wm_page.get/create/list, wm_source.add/process/complete/list/verify, wm_graph.neighbors, wm_lint.check, wm_validate.check, wm_index.rebuild). CLI counterparts: wm search, wm page get/list, wm graph neighbors, wm lint, wm validate. CLI outputs JSON with --json flag or human-readable by default. Full project structure created on wm init with 7 wiki subdirectories (tasks, specs, concepts, patterns, decisions, howto, reference).
