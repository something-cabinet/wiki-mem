---
title: Unify CLI and MCP search pipelines
type: task
status: done
tags: [from-review, search, cli, mcp]
priority: high
knowns_id: kepndl
spec: specs/unify-cli-and-mcp-search-pipelines
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
