---
title: Research Graphify code-intel edge extraction for wm adoption
type: task
id: "wiki:tasks:research-graphify-code-intel-edge-extraction-for-wm-adoption"
status: todo
priority: medium
tags: [research, graphify, code-intel, edges, tree-sitter]
acceptance_criteria:
  - text: "Document Graphify's edge relation taxonomy (imports, imports_from, calls, indirect_call, inherits, implements, references, re_exports, contains, method, case_of, decorates) and compare against wm-code-intel's current edge types"
  - text: "Document Graphify's cross-file resolution pipeline: _resolve_cross_file_imports, receiver-type inference (per-language), re-export chain tracking, tsconfig alias resolution, workspace package resolution, and how these map provenance (EXTRACTED/INFERRED/AMBIGUOUS) — compare against wm graph_resolver"
  - text: "Evaluate Graphify's language coverage (30+ languages via tree-sitter) vs wm-code-intel (7 languages) and identify which extractors would most benefit this project (Rust and TypeScript are already covered)"
  - text: "Assess adoptable patterns: semantic reference edges (field/parameter_type/return_type/generic_arg/attribute/value contexts), the deferred-import distinction for cycle detection, MinHash dedup, the validate.py schema enforcement pattern, and the disambiguate_ambiguous_candidates path-distance heuristic"
  - text: "Identify whether Graphify's community detection (Leiden clustering via graspologic) and graph analysis (god_nodes, surprising_connections, import_cycles, graph_diff) could enhance wm's wiki+code unified graph"
---

Investigate Graphify's code intelligence and edge extraction approach for potential adoption in wiki-mem's wm-code-intel package.

Graphify (github.com/Graphify-Labs/graphify v8) has a sophisticated code-intel pipeline:

## Edge Extraction (tree-sitter AST)
- 30+ language extractors under graphify/extractors/ (each language gets its own tree-sitter grammar)
- Generic extractor engine (graphify/extractors/engine.py, ~3000+ lines) with per-language hooks
- Rich edge relations: imports, imports_from, calls, indirect_call, inherits, implements, references, re_exports, contains, method, case_of, decorates
- Semantic reference edges with typed contexts: field, parameter_type, return_type, generic_arg, attribute, value
- Cross-file resolution pipeline (graphify/extractors/resolution.py): import path resolution, tsconfig aliases, workspace packages, pnpm globs, re-export chain tracking
- Receiver-type inference per language (Java, C#, Swift, C++, TypeScript, Ruby, Python) for member-call resolution
- Deferred import distinction (dynamic import() marked deferred so import-cycle detection ignores them)

## Edge Provenance (directly comparable to wm)
- EXTRACTED: explicitly stated in source (import statement, direct call)
- INFERRED: reasonable deduction (call-graph second pass, co-occurrence)
- AMBIGUOUS: uncertain, flagged for human review

## Graph Analysis
- Community detection via Leiden algorithm (graspologic)
- God-node detection (high-degree nodes)
- Surprising connections (edges between distant communities)
- Import cycle detection
- Graph diff (compare two graphs)

## Comparison to wm-code-intel
| Aspect | Graphify | wm-code-intel |
|--------|----------|---------------|
| Languages | 30+ | 7 (Rust, TS/TSX, Python, Go, HTML, Svelte) |
| Edge types | 12+ relations with contexts | calls, imports, inherits |
| Resolution | Per-language receiver-type, tsconfig, workspace | Single graph_resolver with re-export chains |
| Provenance | EXTRACTED/INFERRED/AMBIGUOUS | Explicit/Derived/Ambiguous |
| Analysis | Leiden clustering, god-nodes, cycles, diff | None (wiki graph has page-type structure) |
| Dedup | MinHash for near-duplicate detection | Content-hash for incremental indexing |

The branch is feat/graphify-gaps (this campaign already has the Derived edge variant and edges_undirected for code-intel). This task evaluates what to adopt next.