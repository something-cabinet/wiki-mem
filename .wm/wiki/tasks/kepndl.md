---
title: Unify CLI and MCP search pipelines
type: task
status: done
tags: [from-review, search, cli, mcp]
priority: high
id: kepndl
spec: specs/unify-cli-and-mcp-search-pipelines
relates_to:
  - {type: implements, target: wiki:specs:unify-cli-and-mcp-search-pipelines}
acceptance_criteria:
  - text: "A shared wm_core::search::query() function exists and is called by both CLI and MCP search"
  - text: "CLI search returns the same results order as MCP wm_search.query for the same query, and supports --type memory to return memory entries"
  - text: "CLI search uses the engine's pre-built BM25 index and auto-triggers a rebuild + retry when the index is unavailable"
---

# Unify CLI and MCP search pipelines

> **Spec:** `specs/unify-cli-and-mcp-search-pipelines`

> *Imported from Knowns task `kepndl`*

# Unify CLI and MCP search pipelines

## Description


P1 from all three reviews. The CLI search builds an inline BM25 from graph metadata only (title, tags, id — no body content, no memory entries). The MCP search uses the engine's pre-built index with full content, memory, RRF fusion, and metadata enrichment.

This means CLI and MCP give different results for the same query. The CLI can't search memory, and its results are missing body content scoring.

Fix options:
1. Route CLI search through the same wm_search.query MCP handler
2. Extract shared query() function in wm-core that both CLI and MCP call
3. At minimum, add memory index rebuild to CLI search and merge results

Labels: from-review, search, cli, mcp


## Acceptance Criteria
