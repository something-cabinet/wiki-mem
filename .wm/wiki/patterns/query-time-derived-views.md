---
title: 'Pattern: Query-time derived views over stored transposes'
type: pattern
id: wiki:patterns:query-time-derived-views
status: draft
tags:
- pattern
- graph
- simplicity
- data-model
relates_to:
  - {type: references, target: wiki:tasks:remove-stored-reciprocal-edges-and-add-edgesundirected-helper}
---

## Problem

Storing a derived/transposed view of your data drags compensating artifacts. In the wiki graph, reciprocal backlink edges were stored at build time (tagged `Derived`) even though the reverse view is one iterator call away (`edges_directed(idx, Incoming)`). The stored transpose caused:
- double-counted degree in exports (explicit-out + derived-in for the same authored edge)
- a special-case 0.5 ranking weight for the fake edges
- a UI legend tier for a provenance that was an artifact
- a phantom-source bug when the reciprocal target page didn't exist yet (graph/mod.rs:186-199)
- semantic incoherence: the pass only covered body `@wiki` refs, not frontmatter `relates_to`, so the "reciprocal" view was incomplete by construction

## Solution

Delete the stored derived data; compute the view at query time. One helper serves all consumers:

```rust
pub fn edges_undirected(graph, idx) -> Vec<EdgeReference<GraphEdge>> {
    // Outgoing first, then Incoming; self-loops deduped
}
```

Storage keeps only authored truth (explicit directed edges). Every consumer needing the reverse direction calls `edges_undirected` on demand. Centrality/blast-radius consumers that legitimately want inbound-only semantics keep `edges_directed(Incoming)` — the helper is for the *display/neighbors* surface.

## When to Use

- The reverse/derived view is cheap to compute (one graph traversal) and requested at query time anyway
- The derived form is partial or incoherent (covers only a subset of inputs — a stored transpose you can't trust is worse than none)
- Derived storage creates consistency hazards (phantom sources, double counting) or demands reconciliation sets

## When Not to Use

- The derived form is expensive and computed repeatedly (precompute, e.g. degree caches)
- The derived view is genuinely distinct data, not a projection of authored edges

## Related

- @wiki/rules/no-compensating-layers — a stored transpose compensating for "we want to see reverse edges" is a bug report, not a feature
- @task-remove-stored-reciprocal-edges-and-add-edgesundirected-helper