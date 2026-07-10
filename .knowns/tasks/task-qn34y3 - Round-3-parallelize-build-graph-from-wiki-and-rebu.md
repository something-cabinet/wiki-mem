---
id: qn34y3
title: 'Round 3: parallelize build_graph_from_wiki() and rebuild_memory_index_from_dir()'
status: done
priority: medium
labels:
  - rayon
  - parallelism
  - performance
createdAt: '2026-07-09T17:31:32.944Z'
updatedAt: '2026-07-09T17:38:13.445Z'
timeSpent: 0
---
# Round 3: parallelize build_graph_from_wiki() and rebuild_memory_index_from_dir()

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
More involved: (1) build_graph_from_wiki() — parallel per-file parsing, sequential graph construction, (2) rebuild_memory_index_from_dir() — parallel file reads + JSON deserialize.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
<!-- AC:END -->

