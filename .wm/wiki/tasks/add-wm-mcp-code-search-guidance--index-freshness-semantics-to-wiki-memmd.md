---
title: Add WM MCP code-search guidance + index-freshness semantics to WIKI-MEM.md
type: task
id: "wiki:tasks:add-wm-mcp-code-search-guidance--index-freshness-semantics-to-wiki-memmd"
status: todo
priority: medium
spec: agent-guidance-wm-mcp-search
acceptance_criteria:
  - text: "WIKI-MEM.md Tool Usage tells agents to prefer wm_code.search / wm_code.symbols / wm_code.deps over raw grep/glob for code, and wm_search.query / wm_graph.* for wiki, tasks, memory, and relationships"
  - text: "WIKI-MEM.md documents freshness: wiki + code indexes self-refresh (watcher + lazy staleness probe); wm index code is repair/force only; wm_index_rebuild is a forced full rebuild only"
  - text: "Guidance notes typed code edges read .wm/state/code.db and need the code-intel feature, while plain text/symbol/dep queries parse live"
  - text: "No standing rebuild-on-code-change step is added; section stays concise"
  - text: "wm_lint.check reports no new issues and wm_validate.check passes"
relates_to:
  - {type: implements, target: wiki:specs:agent-guidance-wm-mcp-search}
---

Finding: WIKI-MEM.md never tells agents to query source code through the WM MCP code tools, nor explains index freshness — so agents fall back to raw grep/glob and assume a code-index rebuild is required after every code change.

Severity: Medium (guidance gap, no runtime defect).

Deliverable: add one concise "Querying Code and Everything via WM MCP" section to WIKI-MEM.md covering tool preference and the verified freshness model (both indexes self-refresh; `wm index code` = repair/force; `wm_index_rebuild` = forced full rebuild).

Files:
- WIKI-MEM.md