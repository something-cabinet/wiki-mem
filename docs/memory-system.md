# Memory System

> Type: concept | Tags: [memory, indexing, retrieval, bm25, salience]

## Overview

Wiki Memory Engine's memory system stores lightweight, durable knowledge entries separate from wiki pages. Memory entries live as JSON files in `.wm/memory/`, are indexed by their own BM25 index, and participate in cross-entity search alongside wiki pages. Unlike Knowns' 3-layer memory (project/session/global), WM uses a flat file-based model where all memories are equal citizens — scoping is handled by tags and content, not storage tiers.

## Technical Explanation

### MemoryEntry Format

Each memory is a JSON file in `.wm/memory/<id>.json`:

```json
{
  "id": "auth-pattern-jwt",
  "title": "JWT Auth Pattern",
  "content": "Use RS256 for JWT. Short-lived access tokens (15min), long-lived refresh tokens (7 days).",
  "tags": ["auth", "pattern", "security"],
  "created_at": "2026-07-07T00:00:00Z",
  "updated_at": "2026-07-07T00:00:00Z"
}
```

Files are discovered by scanning `.wm/memory/` for `.json` files during `index.rebuild`. ID is derived from the filename (without extension).

The `MemoryEntry` struct is defined in `wm-core/src/engine.rs`:

```rust
pub struct MemoryEntry {
    pub id: String,
    pub title: String,
    pub content: String,
    pub tags: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
}
```

### Memory vs Pages

| Aspect | Wiki Pages | Memory Entries |
|--------|-----------|----------------|
| Storage | `.wm/wiki/**/*.md` with YAML frontmatter | `.wm/memory/*.json` |
| Index | Page BM25 (sections) | Memory BM25 (flat) |
| Graph | Full petgraph node with typed edges | Not in graph |
| Recency | FSRS-6 (tasks only) | Salience boost |
| Use case | Structured knowledge, specs, tasks | Quick recall, patterns, conventions |
| Search type | `"page"` or `"all"` | `"memory"` or `"all"` |

### Memory BM25 Index

The memory index is separate from the page index (`EngineState.memory_index: ArcSwap<Bm25Index>`) but uses the same `Bm25Index` struct. During rebuild:

1. Scan `.wm/memory/*.json` files
2. For each file, deserialize as `MemoryEntry`
3. Create an `IndexedDoc` with fields:
   - `title` (weight 4.0)
   - `tags` (weight 2.2)
   - `content` (weight 1.0)
4. Build the BM25 index
5. Atomically swap via `ArcSwap`

Doc ID format: `memory:<id>` — this prefix distinguishes memory results from page results in cross-entity search.

### Staleness Detection

The memory index uses the same two-tier staleness strategy as the wiki graph:
- **Dirty bit** (`AtomicBool stale_flag`): set on internal writes
- **Directory mtime** (`memory_dir_mtime`): checked on every query for external edits

If the `.wm/memory/` directory mtime changes between queries, the memory index is automatically rebuilt.

### Salience Boost (Not Recency)

Unlike task pages (which use FSRS-6 recency), memory entries use a **salience boost** — a score multiplier that ensures memory entries remain visible without decay:

```rust
adjusted_score = memory_score.max(salience_boost.min(salience_clamp / memory_score))
```

This reflects the design philosophy that **memories represent durable knowledge**, not time-sensitive work items. A memory about "we use repository pattern" should be findable whether it was created yesterday or 6 months ago.

### Comparison: WM vs Knowns 3-Layer Memory

| Feature | Knowns | WM |
|---------|--------|-----|
| Layers | Project / Session / Global | Flat file-based |
| Session memory | Built-in, scoped to AI session | Not supported (ephemeral only) |
| Global memory | Cross-project memory | Not supported |
| Dedicated CLI | `knowns memory add/list/edit` | File-based (create .json in .wm/memory/) |
| MCP tool | `memory` tool | Implicit via `wm_search.query(type="memory")` |
| Search | Part of semantic/keyword search | BM25 with salience boost |

**Gap:** WM lacks Knowns' session-scoped working memory and global cross-project memory. Workaround: session context is captured in wiki task pages; cross-project patterns must be manually copied or sourced from a shared repo.

## Configuration Reference

Memory-specific parameters in `config.json` → `search.scoring`:

```json
{
  "search": {
    "scoring": {
      "memory_salience_boost": 2.0,
      "memory_salience_clamp": 0.1
    }
  }
}
```

## Related Documents

- [Cross-Entity Search](./cross-entity-search.md) — how memories participate in multi-type search
- [ScoringConfig](./scoring-config.md) — memory salience tuning
- [BM25 Search Algorithm](./bm25-search.md) — shared BM25 scoring engine
