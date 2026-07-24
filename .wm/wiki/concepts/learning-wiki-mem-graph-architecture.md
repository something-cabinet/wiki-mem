---
id: wiki:concepts:learning-wiki-mem-graph-architecture
title: Learning: Wiki-Mem Graph Architecture
type: concept
tags: [learning, architecture, graph, wiki-mem]
relates_to:
  - {type: references, target: wiki:tasks:task-awotvr-wiki-graph-engine}
  - {type: references, target: wiki:tasks:task-r8n30s-foundation-mcp-transport}
  - {type: references, target: wiki:tasks:task-g2gckv-bm25-search-onnx-embeddings}
  - {type: references, target: wiki:tasks:task-ifnue0-mcp-tools-initial-search-graph-lint-validate-help-audit-permissions}
---
id: wiki:concepts:learning-wiki-mem-graph-architecture

## Patterns

### ArcSwap Lock-Free Graph
- **What:** Use `ArcSwap<(StableGraph, HashMap<String, NodeIndex>)>` instead of `RwLock<DiGraph>`. Graph and id_index are atomically co-swapped on rebuild. Readers hold `Arc` to the old snapshot and never block on writes.
- **When to use:** Any system with concurrent reads during periodic rebuilds. The rebuild builds a new graph in background, then atomically swaps. Zero reader blocking.
- **Source:** @wiki/tasks/awotvr, @wiki/tasks/r8n30s

### Two-Tier Staleness Detection
- **What:** Atomic boolean `stale_flag` for internal writes (O(1)) + directory mtime check for external edits (O(changed_files)). Handles git pulls and editor saves without file-watching daemons.
- **When to use:** Systems that must detect both internal and external mutations without a background watcher thread.
- **Source:** @wiki/tasks/awotvr

### Code-Aware Two-Pass Tokenizer
- **What:** First pass extracts full identifiers (`ERR_AUTH_401` as a single token), second pass sub-tokenizes on `_` and `-` boundaries. Emits both the full identifier and its components. Preserves technical search queries that standard tokenizers break.
- **When to use:** Any search system over technical/code content where identifiers like `auth-service` or `ERR_AUTH_401` must match exactly.
- **Source:** @wiki/tasks/g2gckv

### Field-Weighted BM25 with Rerank Boosts
- **What:** Custom BM25 with per-field IDF computation (title 4.0, tags 2.2, body 1.0, id 3.0) and rerank boosts (exact title +8.0, id match +7.0, tag match +3.0). Zero-result guard prevents gibberish queries from returning all documents.
- **When to use:** Search over structured documents where field importance varies significantly.
- **Source:** @wiki/tasks/g2gckv

### Topic-Aware Graph Neighbor Scoring
- **What:** `score = edge_weight × BM25_score(query, neighbor_content)`. Combines structural graph priority with topical relevance. Without a query, falls back to edge-weight-only sort.
- **When to use:** Graph traversal where you need relevance-ranked neighbors, not just structural adjacency.
- **Source:** @wiki/tasks/ifnue0

## Decisions

### ArcSwap over RwLock (GOOD_CALL)
- **Chose:** `ArcSwap<(StableGraph, HashMap)>` for graph state
- **Over:** `RwLock<DiGraph>` with write-lock during rebuild
- **Tag:** GOOD_CALL
- **Outcome:** Readers never block. Rebuild is invisible to queries. Zero contention measured.
- **Recommendation:** Use ArcSwap for any state that's read-frequently, written-infrequently.

### Flat Binary Vectors over SQLite (FINAL_DECISION)
- **Chose:** Flat `.wm/state/vectors.bin` binary format (magic `WMV\0`, 200 lines) + `ArcSwap<HashMap>` for reads
- **Over:** SQLite via `rusqlite bundled` 
- **Tag:** GOOD_CALL
- **Outcome:** Simpler format, no schema migrations, no additional C dependency to compile/audit. Vectors are always derivable from wiki pages — zero data loss on crash-corruption. `memmap2` keeps memory low by paging in on demand.
- **Recommendation:** Flat binary wins when data is derivable from source of truth and write frequency is extremely low (rebuild only). SQLite adds compile-time and dependency cost for transactional safety you don't need.

### Custom BM25 over bm25 Crate (GOOD_CALL)
- **Chose:** Custom BM25 with field-weighted scoring + code-aware tokenizer (~300 lines)
- **Over:** `bm25` crate from crates.io
- **Tag:** GOOD_CALL
- **Outcome:** Field weighting is critical for search quality. The crate only supports flat document scoring.
- **Recommendation:** Use crate for simple search. Build custom if you need field weights, incremental IDF, or custom tokenization.

### Prefixed MCP Tools (GOOD_CALL)
- **Chose:** Namespace all MCP tools with `wm_` prefix
- **Over:** Flat tool names like `search.query`, `page.create`
- **Tag:** GOOD_CALL
- **Outcome:** No collisions with host app tools (OpenCode, Kiro may have built-in `code`, `search`, `page` tools).
- **Recommendation:** Always prefix MCP tools. The spec should mandate this.

### CLI as Test Wrapper, MCP as Product (GOOD_CALL)
- **Chose:** CLI is bootstrap-only (init, serve, model, version). Everything else is MCP-only.
- **Over:** Full CLI parity with all features
- **Tag:** GOOD_CALL
- **Outcome:** Forces users to integrate MCP. CLI exists for testing and bootstrap only.
- **Recommendation:** Don't build a full CLI — it creates a maintenance burden and delays MCP adoption.

### No Time Tracking (TRADEOFF)
- **Chose:** Removed time tracking from core spec
- **Tag:** TRADEOFF
- **Outcome:** Reduced scope by ~8 hours of implementation. Tasks don't have time tracking, but the graph infrastructure supports adding it later.
- **Recommendation:** Time tracking can be added as page frontmatter fields + audit log queries when needed.

## Failures

### Flat Binary Vectors (Three Redesigns → Revered)
- **What went wrong:** Started with flat vectors.bin (mmap), moved to ArcSwap<HashMap>, briefly switched to SQLite + ArcSwap, then reverted back to flat binary. Each redesign required rewriting the storage layer.
- **Root cause:** SQLite recommendation was based on "zero marginal C dependency burden" — but compile time (+30s) and security audit burden are real costs. Vector data is derivable from wiki pages, making transactional crash safety unnecessary.
- **Time lost:** ~3 hours (original redesigns), +1 hour (final revert)
- **Prevention:** Prefer simple formats when data is derivable. Only add database dependencies when you need queryability (WHERE clauses), not just persistence.

### DashMap for id_index
- **What went wrong:** Used `DashMap<String, NodeIndex>` for ID lookups. But on graph rebuild, all NodeIndex values change. DashMap could have dangling references. Switched to `ArcSwap<(StableGraph, HashMap)>` — atomic co-swap ensures snapshot consistency.
- **Root cause:** Didn't realize NodeIndex is positional and invalidates on rebuild.
- **Time lost:** ~1 hour
- **Prevention:** petgraph NodeIndex is positional. On full rebuild, the entire index changes. Always co-swap graph + id_index atomically.

### Phased Rollout Planning
- **What went wrong:** Planned "Core 10 tools in MVP, rest in v1.1." User rejected it — wanted zero-to-hero, all tools at once.
- **Root cause:** Assumed phasing was safer. But the user correctly identified that deferred features create an indefinite "v1.1" that never ships.
- **Time lost:** ~2 hours of spec/planning
- **Prevention:** Ask the user about phasing early. Some prefer staged delivery, others want everything in one shot.