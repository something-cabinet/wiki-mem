---
id: wiki:concepts:memory-system
title: Memory System
type: concept
tags: [memory, indexing, retrieval, bm25, salience]
---

---
id: wiki:concepts:memory-system
title: Memory System
type: concept
tags: [memory, indexing, retrieval, bm25, salience]
---

# Memory System

> Type: concept | Tags: [memory, indexing, retrieval, bm25, salience]

## Overview

Wiki Memory Engine's memory system stores lightweight, durable knowledge entries as **wiki pages** (markdown + YAML frontmatter, `type: memory`) under `.wm/wiki/memory/<slug>.md`, indexed by BM25, participating in cross-entity search alongside wiki pages. The old JSON format (`.wm/memory/*.json`) is **legacy** — migrated to wiki pages by `migrate_old_memory_json` and retained only for session-layer entries. Like Knowns' 3-layer memory (project/session/global), WM supports all three layers:
- **project** — `.wm/wiki/memory/` (default)
- **session** — ephemeral per-session (`is_session` special-cased in `wm_memory`)
- **global** — cross-project `$HOME/.wm/wiki/memory/` via `wm_memory.add(layer=global)` / `wm_memory.promote`

## Technical Explanation

### Memory Page Format

Each memory is a wiki page in `.wm/wiki/memory/<slug>.md`:

```markdown
---
title: JWT Auth Pattern
type: memory
tags: [auth, pattern, security]
status: active
---

Use RS256 for JWT. Short-lived access tokens (15min), long-lived refresh tokens (7 days).
```

Created via `wm_memory.add` → `page::create_page(path="memory/<slug>")`. `wm_memory.promote` copies a project memory to the global layer (`$HOME/.wm/wiki/memory/<slug>.md`), keeping the project copy.

### Memory vs Pages

| Aspect | Wiki Pages | Memory Entries |
|--------|-----------|----------------|
| Storage | `.wm/wiki/**/*.md` with YAML frontmatter | `.wm/wiki/memory/*.md` (type: memory) |
| Index | Page BM25 (sections) | Memory BM25 (flat) |
| Graph | Full petgraph node with typed edges | In graph (memory page nodes) |
| Recency | FSRS-6 (tasks only) | Salience boost |
| Use case | Structured knowledge, specs, tasks | Quick recall, patterns, conventions |
| Search type | `"page"` or `"all"` | `"memory"` or `"all"` |

### Memory BM25 Index

The memory index is separate from the page index (`EngineState.memory_index`) but uses the same `Bm25Index` struct. Rebuild scans the memory pages, deserializes frontmatter as `MemoryEntry`, builds `IndexedDoc` with `title` (4.0), `tags` (2.2), `content` (1.0), and swaps atomically.

Doc ID format: `memory:<id>` — distinguishes memory results in cross-entity search.

### Staleness Detection

Same two-tier strategy as the wiki graph:
- **Dirty bit** (`stale_flag`) — set on internal writes
- **Directory mtime** — checked on query for external edits

### Salience Boost (Not Recency)

Memories use a **salience boost** rather than FSRS-6 recency — durable knowledge stays findable regardless of age.

### Comparison: WM vs Knowns 3-Layer Memory

| Feature | Knowns | WM |
|---------|--------|-----|
| Layers | Project / Session / Global | Project / Session / Global |
| Session memory | Built-in | Ephemeral via `wm_memory.add(layer=session)` |
| Global memory | Cross-project | `$HOME/.wm/wiki/memory/` via `layer=global` / `promote` |
| Dedicated CLI | `knowns memory add/list/edit` | `wm_memory` tool |
| MCP tool | `memory` tool | `wm_memory` tool with layer param |
| Search | Semantic/keyword | BM25 with salience boost across all layers |

## Configuration Reference

Memory parameters in `config.json` → `search.scoring`:

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

- [Cross-Entity Search](./cross-entity-search.md)
- [ScoringConfig](./scoring-config.md)
- [BM25 Search Algorithm](./bm25-search.md)
- Failure: `wm_memory.promote` path bug (fixed — double `memory/` append, `$HOME` resolution, stale-graph read)