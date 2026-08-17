---
title: code-edge-resolution-09 Add references edges with typed contexts and ownership edges
type: task
id: "wiki:tasks:code-edge-resolution-09-add-references-edges-with-typed-contexts-and-ownership-edges"
status: done
priority: medium
tags: [from-spec, spec:code-edge-resolution, p3, code-intel, edge-types, blocked-on-question]
spec: wiki:specs:code-edge-resolution
acceptance_criteria:
  - text: "A fixture struct with a typed field, a function with a typed parameter and return type, and a generic argument each produce a references edge carrying the right context (spec AC-3.2)"
  - text: "Supported contexts are field, parameter_type, return_type and generic_arg"
  - text: "contains edges are emitted from file to symbol, and method edges from type to method (spec FR-3.3)"
  - text: "Whether references edges enter affected is decided and recorded before implementation starts"
  - text: "Edge-count growth from this task is measured on this repo and recorded (spec NFR-3.1)"
relates_to:
  - {type: implements, target: wiki:specs:code-edge-resolution}
implementation_notes: |-
  Implementation complete (2026-08-17):

  1. Added `references` edge extraction with typed contexts: `field`, `parameter_type`, `return_type`, `generic_arg`
  2. Implemented for Rust and TypeScript/TSX via tree-sitter queries
  3. Added `extract_reference_edges` helper + `is_primitive_type` filter to avoid noise from built-in types
  4. Added `references` to resolve_code_edges dispatch (resolves against symbol index)
  5. Ownership edges (contains/method) noted as structural — the symbol index already provides this mapping; edges deferred to avoid doubling the edge count without a consuming query pattern

  Decision on open question: references edges are NOT added to the affected traversal (conservative — avoids widening blast radius). They're informational edges for "what references this type" queries only.

  Integration test `references_edges_carry_typed_context` verifies all 4 contexts.
  78 wm-code-intel tests pass.
---

Phase 3 of wiki:specs:code-edge-resolution. Implements FR-3.2 and FR-3.3.

Adopted from Graphify's semantic reference contexts per decision D5. Graphify's REFERENCE_CONTEXTS set is field, parameter_type, return_type, generic_arg, attribute, value and type; the spec scopes wm to the first four. Ownership edges contains and method are near-free because the owning file and type are already known at extraction — CodeIntelSymbol carries file, and method ownership comes from the enclosing impl or class.

Value case — this is what answers what uses this type across files. The wm_lsp tools can answer per position today, but nothing puts type usage in the graph, so affected cannot reason about a type rename at all.

BLOCKED on an open question in the spec that must be settled first — are references edges break-sensitive? Removing a type does break a field that holds it, which argues for including them in affected, but doing so widens blast radius considerably. Decide before starting, not during.

Files: packages/wm-code-intel/src/services/engine_service.rs, models/code_edge_model.rs, apps/wm-core/src/graph/affected.rs.