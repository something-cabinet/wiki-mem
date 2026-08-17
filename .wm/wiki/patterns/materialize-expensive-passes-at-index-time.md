---
title: 'Pattern: Materialize expensive passes at index time'
type: pattern
id: wiki:patterns:materialize-expensive-passes-at-index-time
status: draft
tags: [pattern, architecture, performance, code-intel]
relates_to:
  - {type: references, target: wiki:tasks:code-edge-resolution-03-materialize-resolved-edges-in-codedb}
---

## Problem

A global analysis pass (e.g. symbol resolution, type inference, import graph construction) is expensive: it must read all indexed symbols, build lookup structures, and iterate all edges. Running it on every query makes the query path O(index-size) instead of O(result-size), creating latency that compounds at tool-call frequency.

## Solution

**Run the expensive pass once at index time and persist the results.** Query paths read pre-computed data from the store.

Implementation pattern:
1. After raw data is written to the index (e.g. `bulk_upsert_files`), run the global pass.
2. Persist the computed results in a dedicated table/store (e.g. `resolved_edges` table).
3. Query paths check for materialized data first, falling back to on-the-fly computation only when the materialization hasn't run yet.

```rust
// Index-time: materialize after ingest
pub fn rebuild_code_index(db, project_root, force) -> Result<Stats> {
    // ... extract and upsert raw data ...
    materialize_resolved_edges(db, Some(project_root))?;
    Ok(stats)
}

// Query-time: read pre-materialized, fallback to compute
pub fn load_code_graph(project_root) -> Result<Option<Arc<CodeEdgeGraph>>> {
    if db.has_resolved_edges()? {
        return Ok(Some(Arc::new(CodeEdgeGraph::build(db.load_resolved_edges()?))));
    }
    // Fallback: resolve on-the-fly (no materialized edges yet)
    let snapshot = CodeIndexSnapshot::from_db(&db)?;
    Ok(Some(Arc::new(CodeEdgeGraph::build(resolve_code_edges(&snapshot)))))
}
```

Key properties:
- **Incremental-safe:** The index rebuild is already incremental (only changed files reparsed). The global pass re-runs fully because resolution is global (a change in file A can affect resolution of edges in file B). This is acceptable because resolution is fast relative to parsing.
- **Schema migration:** Add the materialized table idempotently (CREATE TABLE IF NOT EXISTS). Old DBs open fine and fall back to on-the-fly.
- **Deterministic:** Same inputs → byte-identical output (NFR-2.1).

## When to Use

- A global analysis pass runs over the full index (cross-file resolution, type inference, cycle detection)
- The analysis is deterministic and local (no network, no LLM)
- Query paths are called at tool-call frequency (many times per agent session)
- The index is updated less frequently than it's queried

## When Not to Use

- The analysis is cheap enough to run per query (e.g. a simple filter or lookup)
- The pass requires data not available at index time (e.g. user query context)
- The index changes as frequently as it's queried (materialization cost > savings)

## Related

- @wiki/specs/code-edge-resolution (D2, NFR-2.2 — resolution runs at index time)
- @wiki/patterns/refresh-derived-state-at-write-path (complementary — when to trigger the re-materialization)