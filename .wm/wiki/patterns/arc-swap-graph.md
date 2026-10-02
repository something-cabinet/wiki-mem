---
title: arc swap graph
id: wiki:patterns:arc-swap-graph
type: pattern
relates_to:
  - {type: example_of, target: wiki:concepts:graph-architecture}
  - {type: implements, target: wiki:specs:graph-engine}
---

schema_version: 1
state: |-
  id: wiki:patterns:arc-swap-graph

  ## When to use

  Any system with concurrent readers during periodic full rebuilds. The graph is read frequently (every query) but rebuilt rarely (on page changes). Standard RwLock blocks all readers during rebuild. ArcSwap eliminates blocking entirely.

  ## How it works

  ```rust
  type GraphSnapshot = Arc<(StableGraph<WikiPageMeta, EdgeType>, HashMap<String, NodeIndex>)>;
  pub graph: ArcSwap<GraphSnapshot>,
  ```

  1. Background task builds the new graph + id_index in a thread
  2. On completion, `ArcSwap::store(Arc::new(new_snapshot))` — atomic pointer swap
  3. Existing readers hold `Arc::clone()` of the old snapshot — they finish with consistent data
  4. New readers automatically see the new snapshot
  5. The old snapshot is dropped when the last reader releases it

  **Critical:** The graph and its id_index must be co-swapped atomically. If you swap them independently, readers can get a graph with a stale index (dangling NodeIndex references).

  ## Example

  From wiki-mem's `graph.rs`:
  ```rust
  pub fn rebuild_snapshot(graph_swap: &ArcSwap<GraphSnapshot>, wiki_dir: &Path) -> usize {
      let (graph, id_index) = build_graph_from_wiki(wiki_dir);
      let count = graph.node_count();
      let snapshot = Arc::new((graph, id_index));
      graph_swap.store(snapshot);
      count
  }
  ```

  ## Source
  @wiki/tasks/awotvr
questions:
  - id: problem_kind
    type: choice
    instructions: What kind of problem does this pattern solve?
    options:
    - architecture
    - api-design
    - data-model
    - error-handling
    - performance
    - testing
    - ui
    - tooling
    - workflow
  - id: preconditions_required
    type: noul
    instructions: This pattern requires specific preconditions to be met.
  - id: complexity
    type: score
    instructions: How complex is applying this pattern?
    levels:
    - trivial
    - simple
    - moderate
    - complex
  - id: language_specific
    type: noul
    instructions: This pattern is specific to a programming language.
answers: {}
