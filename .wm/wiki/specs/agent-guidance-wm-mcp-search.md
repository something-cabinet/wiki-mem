---
title: Agent Guidance — Prefer WM MCP for Code & Everything
type: spec
id: "wiki:specs:agent-guidance-wm-mcp-search"
status: draft
tags: [docs, steering, code-intel, mcp]
relates_to:
  - {type: relates_to, target: wiki:tasks:add-wm-mcp-code-search-guidance--index-freshness-semantics-to-wiki-memmd}
---

# Agent Guidance — Prefer WM MCP for Code & Everything (with index-freshness semantics)

## Overview

The canonical steering file (`WIKI-MEM.md`) tells agents that tools use the `wm_` prefix but never says to query **source code** through the WM MCP code tools, and never explains index freshness. Two consequences: agents fall back to raw grep/glob, and agents assume a manual code-index rebuild is required after every code change.

This spec closes the guidance gap in `WIKI-MEM.md` and pins the correct freshness model, verified against code.

## Background (finding)

- Wiki index is self-maintaining: `.wm/wiki/` watcher (create/modify → `handle_file_change`, remove → `handle_file_delete`); unified search rebuilds when `stale_flag` is set or BM25 is empty; graph/validate MCP tools call `refresh_all_if_stale`.
- Code index self-maintains for typed edges: the watcher also watches code roots and calls `refresh_code_index_for` on code create/modify/remove; typed-edge reads probe via `refresh_if_stale`; engine startup refreshes too.
- Most `wm_code.*` tools never read `code.db`: plain `wm_code.search` is a live `WalkDir` + regex; `wm_code.symbols` and `wm_code.deps` parse live per file with tree-sitter. Only `wm_code.search` with `edge_type` (and code-aware `wm_graph.neighbors`/`affected`) reads `.wm/state/code.db`.
- Therefore no standing rebuild step is warranted on code change.

## Requirements

- FR-1: `WIKI-MEM.md` instructs agents to prefer `wm_code.search` / `wm_code.symbols` / `wm_code.deps` over raw grep/glob for source code, and `wm_search.query` / `wm_graph.*` for wiki pages, tasks, memory, and relationships.
- FR-2: `WIKI-MEM.md` states the freshness model: the wiki index and the code index self-refresh (file watcher + lazy staleness probe); `wm index code` is for repair/force only; `wm_index_rebuild` is a forced full rebuild only.
- FR-3: Guidance notes that typed code edges (`wm_code.search` with `edge_type`; code-aware `wm_graph.*`) read `.wm/state/code.db` and require the `code-intel` feature, while plain text search and symbol/dep queries parse live.
- FR-4: Guidance must not add a routine "rebuild after every code change" step.

## Non-Functional

- Keep `WIKI-MEM.md` concise; one section, not a new doc.
- Wording must stay true to compiled behavior (no aspirational claims).

## Acceptance Criteria

- [ ] AC-1: `WIKI-MEM.md` contains a section stating code/everything queries go through WM MCP tools.
- [ ] AC-2: Freshness semantics documented — self-refresh by default; manual rebuild only to repair/force.
- [ ] AC-3: No instruction tells agents to rebuild routinely on code change.
- [ ] AC-4: `wm_lint.check` and `wm_validate.check` pass after the doc edit.

## References

- @wiki/specs/wiki-mem-alignment — FR-6: "prefer MCP, CLI as fallback" (related, broader)
- @wiki/specs/canonical-steering-alignment — establishes WIKI-MEM.md as canonical
- Files: `WIKI-MEM.md`; `apps/wm-core/src/engine/main_engine_factory.rs:273-344`; `apps/wm-core/src/graph/code_edges.rs:38`; `apps/wm-core/src/mcp/tools/code.rs:126-338`; `apps/wm-core/src/search/query.rs:96`