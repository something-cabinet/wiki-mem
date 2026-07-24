---
id: wiki:specs:vector-storage
title: Vector Storage
type: spec
status: approved
tags: [vectors, storage, search, knowns-parity]
---
id: wiki:specs:vector-storage

## Overview

Replace `vectors.bin` with a SQLite-backed vector database (using the `turso` Rust crate) for embedding vector storage and search. BM25 stays in-memory.

## Locked Decisions

- D22: Search queries route through HTTP API (async) — turso never called from sync MCP handlers
- D23: Turso replaces vectors.bin only. BM25 stays in-memory.
- D24: wm-vectors-bin removed entirely once turso is live

## Requirements

### FR-1: Chunks table
```sql
CREATE TABLE chunks (
    id TEXT PRIMARY KEY,
    type TEXT NOT NULL,
    content TEXT NOT NULL,
    embedding BLOB,
    token_count INTEGER DEFAULT 0
);
```

### FR-2: Content hashes table
```sql
CREATE TABLE content_hashes (
    source_id TEXT PRIMARY KEY,
    hash TEXT NOT NULL
);
```

### FR-3: Index rebuild writes to turso
On `wm_index.rebuild`, write all page sections + embeddings to turso.

### FR-4: Search queries turso for vectors
`wm_search.query` in hybrid mode queries BM25 (in-memory) + turso (vectors), then RRF fuses.

### FR-5: Incremental updates
On page create/update, upsert the page's chunks into turso.

## Acceptance Criteria
- [ ] AC-1: `turso` crate compiles in wm-core (pure Rust, no C compiler)
- [ ] AC-2: Index rebuild writes vectors to turso `.db` file
- [ ] AC-3: Hybrid search returns results from BM25 + turso vectors
- [ ] AC-4: No `wm-vectors-bin` dependency remains
- [ ] AC-5: All existing tests pass
