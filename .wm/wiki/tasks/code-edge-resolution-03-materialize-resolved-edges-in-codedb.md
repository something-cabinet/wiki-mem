---
title: code-edge-resolution-03 Materialize resolved edges in code.db
type: task
id: "wiki:tasks:code-edge-resolution-03-materialize-resolved-edges-in-codedb"
status: done
priority: high
tags: [from-spec, spec:code-edge-resolution, p2, code-intel, materialization]
spec: wiki:specs:code-edge-resolution
acceptance_criteria:
  - text: "Resolved code edges are persisted in code.db, carrying target file, target symbol, line, provenance and the via chain"
  - text: "Resolution runs at index time, not per query (spec NFR-2.2) — a query path performs no resolution pass"
  - text: "Both graph consumers read materialized edges — apps/wm-core/src/mcp/tools/graph.rs and apps/wm-server/src/routes/graph.rs"
  - text: "Materialized edges survive a process restart and are reused without recomputation"
  - text: "Incremental behaviour is preserved — a single-file edit updates only that file's resolved edges"
relates_to:
  - {type: implements, target: wiki:specs:code-edge-resolution}
implementation_notes: |-
  Implementation complete (2026-08-17):

  1. Added `resolved_edges` table to code.db schema (code_index_db.rs)
  2. Added `replace_resolved_edges()`, `load_resolved_edges()`, `has_resolved_edges()`, `count_resolved_edges()` to CodeIndexDb
  3. Added `materialize_resolved_edges()` public function in ingest_service.rs
  4. Modified `rebuild_code_index()` to call materialization after ingest
  5. Modified `load_code_graph()` in code_edges.rs to read pre-materialized edges (fallback to on-the-fly resolution when table is empty)
  6. Both graph consumers (wm-core MCP + wm-server routes) read via `load_code_graph()` — covered
  7. 3 new tests: round-trip, idempotent replace, survive restart

  All 65 wm-code-intel tests pass. Full workspace build clean.
---

Phase 2 foundation for wiki:specs:code-edge-resolution, decision D2 and NFR-2.2.

Today resolve_code_edges runs per query against a freshly built snapshot and the result is discarded — CodeEdgeGraph::build constructs seven HashMaps, answers one question, and is dropped. Resolution must run once at index time with resolved edges persisted in code.db, so every consumer reads a materialized result.

Sequenced before tasks 04 through 07 so each subsequent resolver improvement lands in a materialized store rather than a per-call rebuild.

Blast radius to watch: this changes the code.db schema, and code edges are read by two independent graph implementations. Unifying those twins is out of scope here and belongs to wiki:tasks:core-server-graph-twin-parity-p2s-and-wmdoc-write-action-output-docs, which should resolve as delete-one rather than keep-in-sync.

Rejected alternative from wiki:reference:graphify-adoption-assessment — Graphify's whole-graph graph.json rewrite. wm's incremental content-hash indexing is the stronger model and must be preserved.

Files: packages/wm-code-intel/src/services/code_index_db.rs, services/graph_resolver.rs, apps/wm-core/src/graph/code_edges.rs.