---
id: wiki:concepts:graph-edge-types-traversal
title: Graph Edge Types and Traversal
type: concept
tags: [graph, edges, traversal, bfs, petgraph]
relates_to:
  - {type: references, target: wiki:specs:graph-edge-arrows}
  - {type: references, target: wiki:specs:obsidian-graph-view}
---
id: wiki:concepts:graph-edge-types-traversal

# Graph Edge Types and Traversal

> Type: concept | Tags: [graph, edges, traversal, bfs, petgraph]

## Overview

Wiki Memory Engine models all project knowledge as a typed directed graph using `petgraph::StableGraph<WikiPageMeta, EdgeType>`. Nodes are wiki pages (tasks, specs, concepts, patterns, decisions, howtos, references). Edges are **typed relationships** with intrinsic priorities that control traversal, neighbor ordering, context assembly, and topic-aware search.

## Technical Explanation

### All 9 Edge Types (+ custom)

There are 9 built-in edge types plus one extensible variant. Each carries a priority (0-10) used for traversal weighting:

| Edge Type | Priority | Semantic | Example |
|-----------|----------|----------|---------|
| `extends` | 10 | Specialization / subclass | `oauth2 extends auth` |
| `implements` | 9 | Concrete realization | `endpoint implements spec` |
| `part_of` | 8 | System composition | `login part_of auth-system` |
| `supersedes` | 8 | New version replacing old | `auth-v2 supersedes auth-v1` |
| `example_of` | 6 | Concrete illustration | `sample example_of pattern` |
| `depends_on` | 5 | Prerequisite dependency | `token depends_on user-identity` |
| `answers` | 5 | Response to question | `decision answers spec` |
| `references` | 1 | General citation | `howto references concept` |
| `relates_to` | 0 | Generic (weakest) | `task relates_to concept` |
| `custom("<name>")` | 0 | Domain-specific | `custom("authenticates")` |

### Custom Edge Types

Custom edge types must be **pre-registered** in `config.json` → `custom_edge_types: ["authenticates", "delegates_to"]`. Unregistered custom types are rejected at graph build time with a warning. This prevents typos and ensures the edge taxonomy stays intentional.

### Declaring Edges (Frontmatter)

Edges are declared in YAML frontmatter using the `relates_to` mapping format:

```yaml
relates_to:
  - {type: extends, target: wiki:concepts:graph-architecture}
  - {type: implements, target: wiki:specs:graph-engine}
  - {type: depends_on, target: wiki:concepts:bm25-search}
```

Wikilinks in page body (e.g. `page-id`) are automatically added as `relates_to` edges.

Internally, edges are stored as colon-delimited strings (`"extends:wiki:concepts:base-auth"`) and parsed using `split_once(':')` during graph rebuild.

### Graph Structure

The graph is an `ArcSwap<(StableGraph, HashMap<String, NodeIndex>)>` — a pair of the petgraph graph and a string-to-index lookup co-swapped atomically. This means:
- Readers never block on writes
- New graph is built in isolation, then atomically swapped
- Queries always see a consistent snapshot

### Cycle Detection

After graph build, `petgraph::algo::is_cyclic_directed` checks for cycles. **Cycles are diagnostic only** — they're reported to `lint.check` and logged to stderr, but the graph is never mutated. BFS traversal uses a `HashMap<String, bool>` for visited tracking to prevent infinite loops even in cyclic graphs.

### Traversal Modes

#### 1. `graph.neighbors(id, query?)` — Topic-Aware Neighbor Sorting

Returns neighbors of a page sorted by relevance:
- **With query:** `score = edge_priority × (1.0 + title_relevance)` where title_relevance is 8.0 (exact match), 4.0 (prefix), or 0.0 (none)
- **Without query:** sorted by edge priority descending

Controlled by `graph_depth_neighbors_default` (default 2) and `graph_depth_neighbors_max` (default 5).

#### 2. `graph.path(start, end, max_depth?)` — BFS Shortest Path

Finds the shortest path between two pages using BFS with visited tracking. Returns ordered path with edge types.

#### 3. `graph.subgraph(center, depth?)` — Neighborhood Extraction

Extracts the neighborhood around a center node at configurable depth (max 5). Returns nodes and edges as an adjacency list.

#### 4. `search.retrieve(query, token_budget)` — Context Assembly with Token Budget

BFS from match node with content tiers:

1. **Tier 1 (full content):** Match node + high-relevance neighbors (score > 5.0)
2. **Tier 2 (frontmatter+headers):** Medium relevance (score > 2.0)
3. **Tier 3 (title+edge only):** Low relevance (score ≤ 2.0)

Token budget is a hard cap (256–131072 chars). Uses 4:1 character-to-token approximation. Traversal stops when budget is exhausted.

The traversal depth for retrieval is controlled by `graph_depth_retrieve` (default 2) and `graph_depth_retrieve_min_priority` (default 5) — edges below this priority are not traversed during context assembly.

## Configuration Reference

```json
{
  "custom_edge_types": ["authenticates", "delegates_to"],
  "search": {
    "scoring": {
      "graph_depth_rrf": 1,
      "graph_depth_retrieve": 2,
      "graph_depth_retrieve_min_priority": 5,
      "graph_depth_neighbors_default": 2,
      "graph_depth_neighbors_max": 5
    }
  }
}
```

| Parameter | Default | Description |
|-----------|---------|-------------|
| `custom_edge_types` | `[]` | Custom edge type names to allow |
| `graph_depth_rrf` | 1 | Graph expansion depth for RRF reranking |
| `graph_depth_retrieve` | 2 | Max BFS depth for context assembly |
| `graph_depth_retrieve_min_priority` | 5 | Min edge priority to traverse during retrieval |
| `graph_depth_neighbors_default` | 2 | Default neighbor depth |
| `graph_depth_neighbors_max` | 5 | Max allowed neighbor depth |

## Related Documents

- [ScoringConfig](./scoring-config.md) — graph depth parameters
- [BM25 Search Algorithm](./bm25-search.md) — topic-aware scoring integration
- [Cross-Entity Search](./cross-entity-search.md) — graph expansion in multi-type search