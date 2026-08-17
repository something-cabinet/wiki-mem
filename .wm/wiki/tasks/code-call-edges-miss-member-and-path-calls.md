---
title: Code call edges miss member and path calls
type: task
id: "wiki:tasks:code-call-edges-miss-member-and-path-calls"
status: cancelled
priority: high
tags: [bug, code-intel, edges, tree-sitter, graphify]
acceptance_criteria:
  - text: "extract_edges captures member/method calls (Rust field_expression, TS/TSX member_expression, Python attribute) and path calls (Rust scoped_identifier, Go selector) in addition to bare identifiers"
  - text: "A fixture test asserts one call edge per call form: bare fn(), self.method(), Type::assoc(), obj.method() in TS — each RED before the query change"
  - text: "Receiver-type inference maps a local binding to its declared type for Rust and TS so member calls resolve to a single defining file instead of defaulting to Ambiguous"
  - text: "Ambiguous call-edge share does not increase relative to the pre-change baseline (measured on this repo's own index, recorded in the task notes)"
  - text: "wm graph affected on a symbol reached only through a method call returns the caller"
implementation_notes: Superseded by wiki:specs:code-edge-resolution Phase 2 (2026-08-14). Cancelled rather than done — no code has changed. Original scope is now split across wiki:tasks:code-edge-resolution-04-capture-every-call-form-and-the-receiver-expression, wiki:tasks:code-edge-resolution-05-infer-receiver-types-in-the-global-resolution-pass and wiki:tasks:code-edge-resolution-06-path-distance-disambiguation-drop-unresolvable-record-baseline. The measured call-site evidence is preserved in the spec Overview. Task status vocabulary has no superseded value, so cancelled is the closest allowed state.
---

Verified 2026-08-14 while researching Graphify adoption. packages/wm-code-intel/src/services/engine_service.rs extract_edges uses the tree-sitter query (call_expression function: (identifier) @name) for Rust/TS/TSX/Go and (call function: (identifier) @name) for Python. Method calls are a field_expression / member_expression callee and associated/path calls are a scoped_identifier callee, so neither node ever matches an identifier pattern and neither yields a calls edge. Measured on this repo (rg -o over apps/ and packages/, *.rs): 13009 method-call sites, 2912 path-call sites, 3838 bare-identifier call sites (heuristic, inflated by fn declarations). The call graph therefore observes well under a quarter of Rust call sites, and TypeScript Angular code is member-call dominated. Consequence: wm graph affected under-reports blast radius, and FR-2.1/AC-2.1 of wiki:specs:graphify-gap-closure passed only because its fixtures used bare-identifier calls. Graphify solves the resolution half with per-language receiver-type inference (graphify/extractors/resolution.py); without it, capturing member calls would mostly emit Ambiguous edges, which now carry a 0.25x search-ranking penalty. Full analysis: wiki:reference:graphify-adoption-assessment section 2.