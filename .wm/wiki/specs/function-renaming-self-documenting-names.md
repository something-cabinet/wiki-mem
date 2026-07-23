---
title: Function Renaming — Self-Documenting Names in wm-core
type: spec
tags: [spec, refactor, naming]
status: draft
---

# Spec: Function Renaming — Self-Documenting Names

## Overview

Rename 15 ambiguous function names across wm-core that fail the "self-documenting" test — a reader at the call site should know the function's behavior and return type without clicking through.

152 total functions audited. 130 (85%) already pass. 15 need renaming.

## Requirements

### FR-1: Rename ambiguous `query()` → `run_unified_search()`

**File:** `search/query.rs:70`

`query()` is the most ambiguous name in the codebase. It orchestrates BM25 keyword search, semantic vector search, hybrid RRF fusion, memory search, recency boosting, and salience boosting across pages and memory entries.

```rust
// Before
pub fn query(engine: &EngineState, params: &QueryParams) -> Result<Vec<QueryResult>, String>
// After
pub fn run_unified_search(engine: &EngineState, params: &QueryParams) -> Result<Vec<QueryResult>, String>
```

**Callers to update:** `wm-core/src/mcp/tools/search.rs`, `wm-core/src/mcp/tools/index.rs`

### FR-2: Remove debug `hello()` cruft

**File:** `code_intel.rs:606`

A function named `hello()` in production code that isn't called anywhere. Delete it.

### FR-3: Rename `resolve_all()` → `resolve_all_references()`

**File:** `reference.rs:184`

"All of what?" The function resolves all @doc/@task/@memory/@decision/@template references in content.

### FR-4: Rename `process_source()` → `claim_source_and_read_content()`

**File:** `source.rs:69`

"Process" is the canonical ambiguous verb. Does CAS state transition AND reads file content.

### FR-5: Rename `lint_fix()` → `auto_fix_missing_frontmatter()`

**File:** `graph.rs:249`

Both "lint" and "fix" are ambiguous. Auto-fills missing title/type frontmatter.

### FR-6: Rename `rebuild_snapshot()` → `rebuild_graph_snapshot()`

**File:** `graph.rs:178`

"Snapshot" is generic. Clarifies it's the graph snapshot (ArcSwap).

### FR-7: Rename `build_embeddings()` → `rebuild_embeddings_skip_unchanged()`

**File:** `embed.rs:561`

Simple name hides 3-phase incremental rebuild with hash-awareness.

### FR-8: Rename `VectorStore::swap()` → `replace_entries_and_hashes()`

**File:** `embed.rs:305`

"Swap" is directionally ambiguous. Atomically replaces entries + hashes ArcSwaps.

### FR-9: Rename `verify_source()` → `check_source_staleness()`

**File:** `source.rs:195`

"Verify" is vague. Recomputes SHA-256, returns bool for staleness.

### FR-10: Rename `enrich_and_sort()` → `enrich_search_results_from_graph()`

**File:** `search/query.rs:41`

"Enrich" with what? Adds centrality + page type rank from the graph.

### FR-11 to FR-15

| Current | New | File |
|---|---|---|
| `task_board()` | `build_task_board()` | `task.rs:22` |
| `render()` | `render_template()` | `template_engine.rs:31` |
| `rebuild_memory_index()` | `rebuild_memory_index_from_disk()` | `engine/state.rs:155` |
| `MainEngine::mark_stale()` | `flag_all_indexes_stale()` | `engine/main.rs:162` |
| `EngineState::rebuild_memory_index()` | `rebuild_memory_index_from_disk()` | `engine/state.rs:155` |

## Acceptance Criteria

- [ ] AC-1: `run_unified_search()` exists, `query()` removed
- [ ] AC-2: `hello()` removed from `code_intel.rs`
- [ ] AC-3: `resolve_all_references()` exists, `resolve_all()` removed
- [ ] AC-4: All 15 renames applied across the codebase
- [ ] AC-5: All call sites updated (no dead references)
- [ ] AC-6: `cargo check -p wm-core` passes
- [ ] AC-7: `cargo clippy -p wm-core -- -D warnings` passes
- [ ] AC-8: `cargo test -p wm-core` passes (170 tests)

## Technical Notes

- Mechanical rename — grep + replace. No logic changes.
- The `query()` rename is highest-risk (most callers).
- The `hello()` deletion is safe — grep confirms zero callers.
- Old names can be kept as deprecated aliases if needed, but unlikely — internal API only.