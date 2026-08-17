---
title: code-edge-resolution-04 Capture every call form and the receiver expression
type: task
id: "wiki:tasks:code-edge-resolution-04-capture-every-call-form-and-the-receiver-expression"
status: todo
priority: high
tags: [from-spec, spec:code-edge-resolution, p2, code-intel, tree-sitter]
spec: wiki:specs:code-edge-resolution
acceptance_criteria:
  - text: "A fixture per language exercises every call form and each asserts one edge with the correct target and file plus line (spec AC-2.1)"
  - text: "Rust fixtures cover bare fn(), self.method(), Type::assoc() and obj.method()"
  - text: "TypeScript and TSX fixtures cover bare, this.method(), obj.method() and Namespace.fn()"
  - text: "Python fixtures cover bare and obj.attr() calls; Go fixtures cover bare, pkg.Func() and recv.Method()"
  - text: "Every new fixture assertion fails against pre-change code, verified RED before the query change"
  - text: "Extraction records the receiver expression alongside the callee so resolution is not handed a bare name (spec FR-2.2)"
relates_to:
  - {type: implements, target: wiki:specs:code-edge-resolution}
---

Phase 2 of wiki:specs:code-edge-resolution. Implements FR-2.1 and FR-2.2.

extract_edges in packages/wm-code-intel/src/services/engine_service.rs uses the query (call_expression function - (identifier) @name) for Rust, TS, TSX and Go, and (call function - (identifier) @name) for Python. Method calls are a field_expression or member_expression callee and associated or path calls are a scoped_identifier callee, so neither node kind ever matches an identifier pattern and neither produces an edge.

Measured on this repo with rg over apps/ and packages/ for *.rs — 13009 method-call sites, 2912 path-call sites, 3838 bare-identifier call sites where the last figure is a heuristic inflated by fn declarations. Under a quarter of Rust call sites currently produce an edge, and Angular TypeScript is more member-call dominated still.

Spec AC-2.1 was gradeable before only because the graphify-gap-closure fixtures called helper(), a bare identifier. Enumerate call forms in the fixtures so the domain, not the implementation, sets the bar.

Per D1 the scope is all 5 edge-capable languages. HTML and Svelte stay edge-less. Sizing note from the plan check — if this exceeds roughly 5 files, split by language with Rust and TypeScript first.

Files: packages/wm-code-intel/src/services/engine_service.rs, models/code_edge_model.rs, tests/code_edges.rs.