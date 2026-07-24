---
title: Code Index Cache
type: spec
status: reviewed
tags: [code-intel, performance, turso, caching]
relates_to:
  - {type: extends, target: "wiki:concepts:code-intelligence"}
  - {type: implements, target: "wiki:tasks:improve-code-intel-performance"}
  - {type: references, target: "wiki:patterns:hash-skip-rebuild"}
  - {type: references, target: "wiki:decisions:code-index-cache-architecture"}
---

# Code Index Cache

## Overview

Replace walkdir-every-query in `wm_code.symbols` and `wm_code.deps` with turso-backed SQL queries populated by a hash-skip rebuild step. `wm_code.search` remains walkdir + regex (text search is a different problem). A staleness signal prevents silent cache drift.

## Locked Decisions

- D1: **Option 1+3 hybrid** — index-build during `wm_index.rebuild` + hash-cached SQL, no file watcher in v1. (Source: @oracle ses_0714010b8ffeisqhJTfxvQZLsl)
- D2: **Separate `.wm/state/code.db`** — not merged into vectors.db. Different lifecycle (seconds vs minutes rebuild), different concern.
- D3: **Staleness via quick metadata stat** — on query, compare file count + max mtime against cached metadata. Return `"stale": true` hint. Optional `refresh: true` param for blocking incremental re-index of changed files.
- D4: **`wm_code.search` stays as-is** — text regex search on filesystem doesn't benefit from symbol indexing. Will use `code_files` table as a filtered file list to skip unsupported dirs consistently.
- D5: **Rayon-parallel tree-sitter extraction** — same pattern as `rebuild_embeddings_skip_unchanged` in `wm-embed`. Tree-sitter parse is CPU-bound and embarrassingly parallel.

## Requirements

### Functional Requirements

- FR-1: `wm_code.symbols` queries from a pre-built SQLite cache instead of walking + parsing the filesystem on every call
- FR-2: `wm_code.deps` queries from the same cache, supporting the same filter parameters (file, language, reverse, depth)
- FR-3: Both tools return a `stale` boolean + `hint` string when the cached index doesn't reflect the current filesystem state
- FR-4: Both tools accept an optional `refresh: true` parameter that triggers an incremental re-index of changed files before answering
- FR-5: `wm_index.rebuild` includes a code index step (after BM25 + embeddings) that scans supported source files, extracts symbols and deps via tree-sitter, and stores them in `code.db`
- FR-6: `wm index code` standalone CLI command to rebuild only the code index (fast, no BM25/embeddings)
- FR-7: `wm_index.status` reports code_files_indexed count and code_stale boolean
- FR-8: Hash-skip: files with unchanged SHA-256 content are skipped (same pattern as `rebuild_embeddings_skip_unchanged`)

### Non-Functional Requirements

- NFR-1: First `wm_index.rebuild` with code must complete within 2× the time of a tree-sitter parse scan of the entire project
- NFR-2: Subsequent queries after rebuild must return in <50ms regardless of project size
- NFR-3: Incremental refresh (changed files only) must complete in <1s for a typical edit
- NFR-4: Must work identically in MCP daemon mode and one-shot CLI mode
- NFR-5: Must not add new crate dependencies — turso, rayon, tree-sitter, sha2 all already in workspace

## Acceptance Criteria

- [ ] AC-1: `wm_code.symbols(name="User")` returns results from cache in <10ms for a 50K-file corpus
- [ ] AC-2: `wm_code.deps(file="src/auth.rs")` returns deps from cache in <10ms
- [ ] AC-3: Editing a file, then calling with `refresh: true` re-indexes only that file and returns updated results
- [ ] AC-4: After rebuild, `wm_index.status` shows `code_files_indexed > 0` and `code_stale == false`
- [ ] AC-5: Adding a new source file without re-running rebuild causes `stale: true` in subsequent query responses
- [ ] AC-6: `wm index code` CLI command rebuilds code index without affecting BM25 or embeddings
- [ ] AC-7: Stale files are detected via mtime/count comparison, not full re-hash (fast stat-only pass)

## Scenarios

### Scenario 1: First rebuild, then query
**Given** no code.db exists yet
**When** user runs `wm index rebuild` (with code step)
**Then** all supported source files are scanned, parsed, and stored in code.db
**When** user calls `wm_code.symbols(name="User")`
**Then** results are returned from SQL in <10ms with `stale: false`

### Scenario 2: Edit file, stale detection
**Given** code.db is populated
**When** user edits `src/auth.rs` (adds a new function)
**Then** `wm_code.symbols()` returns `stale: true` + hint
**When** user calls with `refresh: true`
**Then** only the changed file is re-parsed, results include the new function

### Scenario 3: Reverse deps query
**Given** code.db is populated with all deps
**When** user calls `wm_code.deps(file="src/engine.rs", reverse: true)`
**Then** returns all files that import from engine.rs, from SQL index lookup — no filesystem walking

## Technical Notes

- Follow `VectorDb` pattern in `packages/wm-embed/src/vector_db.rs` for turso sync↔async bridging
- Follow `rebuild_embeddings_skip_unchanged` in `packages/wm-embed/src/lib.rs` for hash-skip + rayon parallelism
- `SKIP_DIRS` constant in `code.rs:4-8` should be reused for filesystem scan
- Use `CodeIntelEngine::is_supported()` from `engine_service.rs:51` for extension filtering
- Side bugs to fix: `code.rs:271,500` file_path.contains false positive; `code.rs:413` depth accepted but ignored

## Open Questions

(none — all locked via Oracle design review)
