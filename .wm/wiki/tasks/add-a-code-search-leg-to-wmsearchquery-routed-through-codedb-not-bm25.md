---
title: Add a code search leg to wm_search.query (routed through code.db, not BM25)
type: task
id: "wiki:tasks:add-a-code-search-leg-to-wmsearchquery-routed-through-codedb-not-bm25"
status: todo
priority: high
tags: [search, code-intel, mcp-tooling, spec-linked]
spec: specs/unified-search-code-results
acceptance_criteria:
  - text: "A code search type returns symbol matches (name and path) and resolved-edge matches from code.db with a distinct result shape (file, symbol, kind, line, edge kind, provenance)"
  - text: "Code results are fused as a separate labelled leg; a test asserts code symbols never appear in the BM25 corpus or affect its field weights"
  - text: "all includes the code leg only when the index is fresh, and a stale or missing index is reported explicitly rather than silently omitted"
  - text: "A code-bearing response states whether the staleness gate ran and the index age"
  - text: "Behaviour without the code-intel feature, and before any index exists, degrades cleanly and states why code results are absent"
  - text: "The emitted MCP tool schema for wm_search_query remains a valid top-level object with the new SearchType variant, verified explicitly"
  - text: "A doc-only query does not run the code leg, shown by timing or by test instrumentation"
  - text: "wiki:specs:agent-guidance-wm-mcp-search is updated to describe unified search and its freshness model"
  - text: "No comments, no #[allow], functions at most 20 lines, tests written first"
---

Implementation task for `@wiki/specs/unified-search-code-results` (draft, awaiting approval).

**Reviewer directive:** search should return code as well as docs, so an agent asking "where is X handled?" gets one answer instead of needing to know a second tool. Chosen approach: a separate code leg reading `code.db`, fused into the ranked list — **code symbols must not enter the BM25 corpus**, because prose retrieval is BM25 plus semantic while code retrieval is exact identifier matching, and mixing them pollutes the doc corpus IDF.

**Verified current state:** `SearchType` is `All | Page | Task | Memory` with no `Code` variant (`apps/wm-core/src/mcp/tools/search.rs:7-12`); the search tool has no code integration; the BM25 corpus is wiki sections only (`apps/wm-core/src/mcp/tools/index.rs:159-166`); `code.db` with materialised `resolved_edges` already exists; the `refresh_if_stale` gate currently covers only the graph path (`apps/wm-core/src/graph/code_edges.rs:38`).

**Coordination, not conflict:** `wiki:specs:agent-guidance-wm-mcp-search` (draft, untracked) addresses the discoverability half by documenting the `wm_code.*` tools. This task **extends** that spec (guidance becomes "search covers code, here is the freshness model") instead of superseding it. It also depends on `wiki:specs:code-index-cache-consumption`, since a code leg is only as good as the cache read path behind it.

**Ordering:** implement after migration step 3 of `wiki:tasks:controlled-rewrite-of-appswm-clisrcmainrs--one-architecture-decision-per-review` lands, to avoid colliding with the CLI rewrite.