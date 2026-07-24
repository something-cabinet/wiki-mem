---
id: wiki:decisions:code-index-cache-architecture
title: Decision: Code Index Cache Architecture
type: decision
status: approved
tags: [code-intel, caching, turso, architecture]
relates_to:
  - {type: references, target: "wiki:patterns:hash-skip-rebuild"}
---
id: wiki:decisions:code-index-cache-architecture

# Decision: Code Index Cache Architecture

## Context

`wm_code.symbols` and `wm_code.deps` used `walkdir::WalkDir` + full file read + regex/tree-sitter parse on every MCP tool call. At project scale (50K+ files), this took minutes per query. Knowns faced the same problem and iterated through three approaches: ingest (tree-sitter→DB, stale after edits), LSP on-demand (cold cache misses, server dependency), and the current LSP-only model. WM needed a solution that works identically in CLI and MCP modes with no daemon dependency.

## Decision

Use an Option 1+3 hybrid — index-build during `wm_index.rebuild` + hash-cached SQL queries, with a staleness signal instead of a file watcher or auto-refresh.

- New `.wm/state/code.db` (turso/SQLite), separate from `vectors.db` — different lifecycle (seconds vs minutes rebuild)
- SHA-256 hash-skip for unchanged files (same pattern as `wm-embed`'s `rebuild_embeddings_skip_unchanged`)
- Queries are SQL — <10ms vs minutes for walkdir
- Staleness detected via fast metadata stat pass (count + max mtime)
- Response includes `stale: true` + hint when index is out of date
- Optional `refresh: true` param for inline re-index of changed files only
- No file watcher — the staleness signal makes it unnecessary (there's always a detectable window)
- Falls through to original walkdir+regex when `code.db` is missing or `code-intel` feature disabled

## Rationale

1. **CLI-offline constraint** rules out watcher-only approaches — stateless one-shot CLI calls must work correctly.
2. **Existing patterns** — turso upsert, SHA-256 hash-skip, rayon parallel extraction were all proven in the wm-embed crate.
3. **Staleness beats silence** — the old walkdir approach was always fresh but painfully slow. The cache returns a stale flag when the index lags, so agents never silently trust stale data.
4. **No new deps** — turso, rayon, sha2, tree-sitter were already in the workspace.
5. **Depth-1 recursion only** — recursive CTE for transitive deps was dropped after Oracle review revealed `code_deps.target` contains module paths (e.g. `crate::engine::EngineState`), not file paths — the CTE could never advance past level 1.

## Consequences

- `wm index rebuild` now takes longer (adds code-index step after embeddings).
- First query after rebuild is instant (<10ms). Subsequent queries check staleness (fast stat pass, cached every 5s).
- Staleness stat walk itself may exceed 10ms at 50K files — spec-internal tension between AC-1 and D3, tracked as follow-up.
- `wm_code.search` (text regex) kept as walkdir + regex — text search is a different problem from symbol/dep lookup.

## Related
- @wiki/specs/code-index-cache
- @wiki/patterns/hash-skip-rebuild
- @wiki/patterns/hash-skip-rebuild
