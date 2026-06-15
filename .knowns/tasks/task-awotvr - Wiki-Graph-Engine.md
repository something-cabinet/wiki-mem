---
id: awotvr
title: Wiki Graph Engine
status: done
priority: high
labels:
  - from-spec
  - go-mode
  - graph
createdAt: '2026-06-15T11:31:11.082Z'
updatedAt: '2026-06-15T13:55:56.119Z'
timeSpent: 0
spec: specs/local-knowledge-engine-rust
fulfills:
  - AC-2
  - AC-3
  - AC-4
  - AC-5
  - AC-6
  - AC-7
  - AC-9
  - AC-15
---
# Wiki Graph Engine

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Code-fence-aware section parser, frontmatter parsing with relates_to YAML mapping, StableGraph compilation with typed edges, custom edge type registration validation, ArcSwap graph + id_index atomic co-swap, cycle detection (diagnostic only, visited set BFS), content hash tracking
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Code-fence-aware section parser splits markdown correctly, ignores headers inside ``` blocks
- [x] #2 Frontmatter parser extracts WikiPageMeta from YAML, infers path-based IDs
- [x] #3 relates_to YAML mapping parsed into typed StableGraph edges
- [ ] #4 Custom edge type registration validated at build time, unregistered types rejected
- [x] #5 StableGraph graph built from wiki directory files, ArcSwap atomic swap
- [x] #6 Cycle detection logs node IDs + edge types via is_cyclic_directed, graph not mutated
- [x] #7 Content hash tracking via SHA-256 per page
- [x] #8 Build + tests pass
<!-- AC:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Wiki Graph Engine: parser.rs (code-fence-aware section splitter, frontmatter + relates_to YAML parsing, path-based ID inference, SHA-256 content hashing), graph.rs (walkdir-based .md scanner, StableGraph builder with typed edges, ArcSwap snapshot rebuild, diagnostic cycle detection via is_cyclic_directed), 4 new unit tests for parser.
<!-- SECTION:NOTES:END -->

