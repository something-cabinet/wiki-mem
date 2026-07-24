---
id: wiki:patterns:arcswap-copy-on-write-incremental
{}
relates_to:
  - {type: references, target: wiki:tasks:p5a-add-notify-file-watcher-to-engine-startup}
---
id: wiki:patterns:arcswap-copy-on-write-incremental

---
id: wiki:patterns:arcswap-copy-on-write-incremental
title: Pattern: ArcSwap Copy-on-Write for Incremental Index Updates
type: pattern
tags: [pattern, graph, architecture]
---
id: wiki:patterns:arcswap-copy-on-write-incremental

## Problem

In-memory indices (graph, BM25, section corpus) use `ArcSwap<Data>` for lock-free reads. Every mutation requires a full rebuild of the entire dataset from scratch — even for single-page changes. This makes page create/update/delete expensive.

## Solution

Use copy-on-write on the existing `ArcSwap` pattern:

1. Load current `Arc<Data>` 
2. Clone the inner data (O(n) but n is small — <1000 nodes)
3. Mutate the clone (add/remove one element)
4. Store new `Arc<Data>` via `ArcSwap::store()` — atomic swap

No readers are blocked. Existing `Arc` snapshots continue working for concurrent readers.

## When to Use

- In-memory indices with <10,000 elements
- Read-heavy workloads (writes are rare, reads are frequent)
- When full rebuild from scratch is wasteful for single-element changes

## When Not to Use

- Large datasets (>100k elements) where clone cost is prohibitive
- Write-heavy workloads where the clone-on-write overhead dominates
- Persistent data that needs transactional integrity (use SQLite instead)

## Implementation

```rust
let snapshot = graph_swap.load();                       // 1. Load current Arc
let mut new_graph = (*snapshot).clone();                 // 2. Clone
new_graph.add_node(meta);                                // 3. Mutate
let mut new_index = index.clone();
new_index.insert(id, node_idx);
graph_swap.store(Arc::new((new_graph, new_index)));      // 4. Atomic swap
```

## Related
- @wiki/specs/graph-connectivity-fix
- @wiki/tasks/57bca4
- @wiki/tasks/7d3aa1
- @wiki/tasks/b6d2ca