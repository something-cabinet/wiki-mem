---
title: "Spec: Wiki Graph Engine"
type: spec
tags: [graph, architecture, spec]
status: draft
---

## Overview

The graph engine spec covers: typed edge relationships, StableGraph construction from wiki pages, ArcSwap atomic swapping, cycle detection, and content hash tracking. See [ArcSwap Graph](../patterns/arc-swap-graph.md) for the implementation pattern.

## Scope

- **Typed edges**: 16 built-in edge types (`extends`, `implements`, `part_of`, `supersedes`, `supports`, `example_of`, `depends_on`, `required_by`, `mitigates`, `causes`, `contradicts`, `questions`, `answers`, `references`, `similar_to`, `relates_to`) plus custom registered types
- **Graph storage**: `petgraph::StableGraph<WikiPageMeta, EdgeType>` with `ArcSwap` for atomic rebuild
- **Node identity**: String-to-`NodeIndex` lookup co-swapped with graph to prevent dangling indices
- **Cycle detection**: `petgraph::algo::is_cyclic_directed` (diagnostic only — graph never mutated)
- **Traversal**: BFS shortest-path, BFS context assembly with token budget, DFS neighborhood extraction
- **Content tracking**: Per-page SHA-256 hashing for partial rebuild optimization

See [Graph Architecture](../concepts/graph-architecture.md) for the conceptual overview and [ArcSwap Lock-Free Graph](../patterns/arc-swap-graph.md) for implementation details.
