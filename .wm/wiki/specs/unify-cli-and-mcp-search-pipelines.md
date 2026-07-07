---
title: Unify CLI and MCP Search Pipelines
type: spec
tags:
  - spec
  - approved
  - search
  - cli
  - mcp
---

## Overview

The CLI and MCP handlers implement separate search pipelines that produce different results. The CLI builds an inline BM25 from graph metadata only (title, tags, id — no body content, no memory entries). The MCP search uses the engine's pre-built index with full content, memory, RRF fusion, and metadata enrichment. This means CLI and MCP give different results for the same query, and CLI cannot search memory at all.

Unify the two paths so search results are consistent regardless of interface.

## Locked Decisions

- D1: Extract shared `wm_core::search::query()` function called by both CLI and MCP
- D2: Full CLI parity — support `--type`, `--mode`, `--limit`, `--offset`, `--recency` flags
- D3: Auto-rebuild — trigger index rebuild if not ready, then retry search (index is always built on app startup, so this edge case is rare but handled gracefully)

## Requirements

### Functional Requirements

- FR-1: CLI and MCP must produce identical search results for the same query and project state
- FR-2: CLI search must support `--type`, `--mode`, `--limit`, `--offset`, `--recency` flags
- FR-3: CLI search must return results from memory entries, not just pages
- FR-4: CLI search must apply RRF fusion and recency boosting
- FR-5: When pre-built index is not available, CLI must auto-rebuild and retry before returning results

### Non-Functional Requirements

- NFR-1: CLI search latency must not regress (shared function uses pre-built index)
- NFR-2: `cargo build` and `cargo test` pass without new warnings

## Acceptance Criteria

- [ ] AC-1: Shared `wm_core::search::query()` function exists and is called by both CLI and MCP
- [ ] AC-2: CLI `wm search query --type memory` returns memory entries
- [ ] AC-3: CLI `wm search query` returns same results order as MCP `wm_search.query` for same query
- [ ] AC-4: CLI search uses engine's pre-built BM25 index (not inline from graph metadata)
- [ ] AC-5: CLI search supports `--mode keyword|semantic|hybrid` flag
- [ ] AC-6: CLI search supports `--recency` flag to toggle recency boost
- [ ] AC-7: CLI search supports `--offset` for pagination
- [ ] AC-8: CLI search results include recency boost for tasks
- [ ] AC-9: When index unavailable, CLI auto-triggers rebuild and retries search — no user-facing error
- [ ] AC-10: If auto-rebuild completes, search results are returned normally
- [ ] AC-11: All existing CLI and MCP search tests pass

## Scenarios

### Scenario 1: CLI vs MCP result parity
**Given** a project with pages, tasks, and memory entries
**When** user runs `wm search query "topic"` and MCP sends `wm_search.query("topic")`
**Then** both return the same ranked results in the same order

### Scenario 2: CLI memory search
**Given** a project with memory entries
**When** user runs `wm search "topic" --type memory`
**Then** memory entries appear in results alongside page matches

### Scenario 3: Index not rebuilt
**Given** a project that hasn't been indexed yet
**When** user runs `wm search query "topic"`
**Then** CLI auto-triggers index rebuild, waits for completion, then returns search results normally

## Technical Notes

- CLI search is in wm-cli/src/main.rs inline code
- MCP search is in wm-core/src/mcp/tools.rs
- Shared function lives in `wm_core::search::query()`
- Returns structured results; CLI formats as text/JSON, MCP returns as JSON-RPC result
- The shared function signature should accept: engine state, query string, optional type filter, optional mode, limit, offset, recency toggle
