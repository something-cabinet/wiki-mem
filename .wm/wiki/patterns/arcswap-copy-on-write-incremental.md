---
---

---
id: wiki:patterns:arcswap-copy-on-write-incremental
{}
relates_to:
  - {type: references, target: wiki:tasks:57bca4}
  - {type: references, target: wiki:tasks:7d3aa1}
  - {type: example_of, target: wiki:tasks:wire-incremental-bm25--onnx-updates-on-page-crud}
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
2. Clone the inner data (O(n) but n is manageable for in-memory indices)
3. Mutate the clone (add/remove one element)
4. Store new `Arc<Data>` via `ArcSwap::store()` — atomic pointer swap

No readers are blocked. Existing `Arc` snapshots continue working for concurrent readers.

## Concrete Example: BM25 Incremental Update on Page CRUD

This pattern was applied to fix the stale-index problem where page mutations set `stale_flag = true` but required a manual full rebuild. The fix uses copy-on-write to add/remove individual documents from `Bm25Index`:

```rust
fn update_bm25_for_page(engine: &EngineState, page_id: &str, content: &str, file_path: &Path, is_delete: bool) {
    let mut bm25 = Bm25Index::clone(&*engine.bm25_index.load());

    // Remove all existing sections for this page
    let prefix = format!("{}#", page_id);
    let to_remove: Vec<String> = bm25.docs.iter()
        .filter(|d| d.id.starts_with(&prefix))
        .map(|d| d.id.clone())
        .collect();
    for id in &to_remove {
        bm25.remove_document(id);
    }

    if !is_delete {
        let sections = parse_sections(file_path, content);
        for section in &sections {
            bm25.add_document(indexed_doc_from_section(section));
        }
    }

    engine.bm25_index.store(Arc::new(bm25));
    engine.section_corpus.rcu(|old| {
        // Same pattern for the section corpus
        let mut corpus = (**old).clone();
        corpus.retain(|s| !s.section_id.starts_with(&prefix));
        if !is_delete {
            corpus.extend(parse_sections(file_path, content));
        }
        corpus
    });
}
```

See `apps/wm-core/src/page/services/page_crud_service.rs` for the full implementation.

## When to Use

- In-memory indices with <10,000 elements
- Read-heavy workloads (writes are rare, reads are frequent)
- When full rebuild from scratch is wasteful for single-element changes
- Data structures that implement `Clone` (Vec, HashMap, etc.)

## When Not to Use

- Large datasets (>100k elements) where clone cost is prohibitive
- Write-heavy workloads where the clone-on-write overhead dominates
- Persistent data that needs transactional integrity (use SQLite instead)
- Data structures that cannot be cheaply cloned

## Related

- @wiki/specs/graph-connectivity-fix
- @wiki/tasks/57bca4
- @wiki/tasks/7d3aa1
- @wiki/tasks/b6d2ca
- @wiki/tasks/wire-incremental-bm25--onnx-updates-on-page-crud
