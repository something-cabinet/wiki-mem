---
id: wiki:concepts:graph-architecture
title: Graph Architecture
type: concept
tags: [graph, architecture, traversal, edges]
status: reviewed
---
id: wiki:concepts:graph-architecture

## Overview

The Wiki Memory Engine models all project knowledge as a typed directed graph. Every wiki page (task, spec, concept, pattern, decision, howto, reference) becomes a node in a `petgraph::StableGraph<WikiPageMeta, EdgeType>`. Edges represent typed relationships between pages — `extends`, `implements`, `depends_on`, `part_of`, and 13 other built-in types — declared via the `relates_to` YAML frontmatter field. This unified graph model replaces separate per-type stores with a single traversable structure where any entity can be related to any other.

## Graph Storage and Atomic Swapping

The graph is stored as `ArcSwap<(StableGraph<WikiPageMeta, EdgeType>, HashMap<String, NodeIndex>)>` — a pair of the petgraph graph and a string-to-index lookup table that are co-swapped atomically on every rebuild. This design ensures that readers never block on writes: a background task builds the new graph + index in isolation, then performs a single atomic pointer swap via `ArcSwap::store`. Existing readers continue using their `Arc`-cloned snapshot of the old graph, while new readers automatically see the new version. The same ArcSwap pattern is used for the BM25 index and vector registry, providing lock-free reads across all core data structures.

## Traversal Strategies and Edge Declaration

Graph traversal uses BFS for shortest-path queries and context assembly (with configurable depth and minimum edge priority), and DFS for full neighborhood extraction. Topic-aware neighbor scoring combines edge priority with BM25 title relevance to sort related pages by query context. Edges are declared in page frontmatter using the `relates_to` mapping format, which ties pages together declaratively. The `wm_graph.neighbors`, `wm_graph.path`, and `wm_graph.subgraph` MCP tools expose these traversal strategies, while `wm_search.retrieve` uses BFS with a token budget to assemble context packs from the graph neighborhood.
