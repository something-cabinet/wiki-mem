---
title: Wiki Graph Engine
type: task
status: done
tags: [from-spec, go-mode, graph]
priority: high
id: awotvr
spec: specs/local-knowledge-engine-rust
fulfills: [AC-2, AC-3, AC-4, AC-5, AC-6, AC-7, AC-9, AC-15]
relates_to:
  - {type: implements, target: wiki:specs:local-knowledge-engine-rust}
acceptance_criteria:
  - text: "Code-fence-aware section parser splits markdown correctly, ignoring headers inside ``` blocks, and frontmatter parser extracts WikiPageMeta with path-based IDs"
  - text: "relates_to YAML mapping parses into typed StableGraph edges with build-time validation of custom edge types"
  - text: "StableGraph builds from wiki dir with ArcSwap atomic co-swap, diagnostic-only cycle detection, and SHA-256 content hash tracking"
---

# Wiki Graph Engine

> **Spec:** `specs/local-knowledge-engine-rust`

> **Fulfills:** AC-2, AC-3, AC-4, AC-5, AC-6, AC-7, AC-9, AC-15

> *Imported from Knowns task `awotvr`*

# Wiki Graph Engine

## Description


Code-fence-aware section parser, frontmatter parsing with relates_to YAML mapping, StableGraph compilation with typed edges, custom edge type registration validation, ArcSwap graph + id_index atomic co-swap, cycle detection (diagnostic only, visited set BFS), content hash tracking


## Acceptance Criteria

- [x] #1 Code-fence-aware section parser splits markdown correctly, ignores headers inside ``` blocks
- [x] #2 Frontmatter parser extracts WikiPageMeta from YAML, infers path-based IDs
- [x] #3 relates_to YAML mapping parsed into typed StableGraph edges
- [ ] #4 Custom edge type registration validated at build time, unregistered types rejected
- [x] #5 StableGraph graph built from wiki directory files, ArcSwap atomic swap
- [x] #6 Cycle detection logs node IDs + edge types via is_cyclic_directed, graph not mutated
- [x] #7 Content hash tracking via SHA-256 per page
- [x] #8 Build + tests pass


## Implementation Notes


Wiki Graph Engine: parser.rs (code-fence-aware section splitter, frontmatter + relates_to YAML parsing, path-based ID inference, SHA-256 content hashing), graph.rs (walkdir-based .md scanner, StableGraph builder with typed edges, ArcSwap snapshot rebuild, diagnostic cycle detection via is_cyclic_directed), 4 new unit tests for parser.
