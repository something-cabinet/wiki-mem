---
id: wiki:patterns:hash-skip-rebuild
title: Pattern: Hash-Skip Incremental Rebuild
type: pattern
status: reviewed
tags: [caching, turso, performance, incremental]
relates_to:
  - {type: references, target: "wiki:decisions:code-index-cache-architecture"}
---
id: wiki:patterns:hash-skip-rebuild

# Pattern: Hash-Skip Incremental Rebuild

## Problem

Derived data (embeddings, code symbols, dep graphs) needs to be cached for fast queries. Full regeneration on every change is too slow at scale. The naive approach — full scan + re-parse on every query — takes minutes for 50K files. A file watcher only works while a daemon runs, breaking CLI-mode correctness.

## Solution

Use SHA-256 content hashing + SQLite hash-cache to skip unchanged files, combined with a fast metadata stat pass for staleness detection.

```
Phase 0: Load existing hashes from DB
  existing = db.load_file_hashes()  // path → (sha256, mtime)

Phase 1: Walk filesystem, collect files
  file_infos = walkdir(project_root, SKIP_DIRS, supported_extensions)
  // No parsing yet — just path + extension

Phase 2: Parallel hash-skip (pure CPU, no DB)
  par_iter file_infos.map(|(path, ext)| {
    mtime = metadata(path).modified()
    if matches_existing_hash(path, mtime, existing) → skip
    content = read(path)
    hash = sha256(content)
    if matches_existing(path, hash, mtime, existing) → skip
    symbols = tree_sitter_extract(content, ext)
    deps = tree_sitter_extract_deps(content, ext)
    FileData { path, sha256, mtime, symbols, deps }
  }).collect()

Phase 3: Bulk upsert (single transaction, sequential)
  BEGIN TRANSACTION
  for file_data in changed_files:
    UPSERT code_files
    DELETE + INSERT code_symbols
    DELETE + INSERT code_deps
  COMMIT

Phase 4: Delete stale entries
  known_paths = all_paths_from_phase_1
  DELETE FROM code_files WHERE path NOT IN known_paths
```

## Key Properties

- **Crash-safe** — BEGIN/COMMIT wraps all writes. Partial rebuild does not corrupt the index.
- **O(modified files)** — unchanged files resolve in O(1) hash lookup. No re-parsing.
- **Parallel-safe** — tree-sitter extraction in rayon `par_iter` is stateless. DB writes are serialized via mutex in a single transaction.
- **Works offline** — no daemon, no watcher, no LSP servers. One-shot CLI calls produce correct results.

## When to Use

- Caching derived data from source files
- Data can be incrementally updated (hash-skip detects changes)
- Query latency must be <10ms for cached data
- Both daemon and CLI modes must work identically

## When Not to Use

- Source of truth is small enough to regenerate on every query (<100 files)
- Data requires real-time freshness (millisecond-level staleness windows)
- Data is not derivable from files (user-generated or API-sourced)

## Related
- @wiki/decisions:code-index-cache-architecture
- @wiki/specs:code-index-cache
- @wiki/concepts:hash-skip-rebuild
