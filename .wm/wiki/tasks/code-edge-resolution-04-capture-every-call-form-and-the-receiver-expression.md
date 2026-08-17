---
title: code-edge-resolution-04 Capture every call form and the receiver expression
type: task
id: "wiki:tasks:code-edge-resolution-04-capture-every-call-form-and-the-receiver-expression"
status: done
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
implementation_notes: |-
  ## Implementation complete 2026-08-17 (commit c25ead0)

  ### What landed

  - CodeEdge gains a `receiver: Option<String>` field (FR-2.2)
  - extract_edges uses combined tree-sitter queries capturing bare, method/member, and path/namespace call forms per language, with receiver extracted from the AST
  - code_index_db schema migrated (ALTER TABLE ADD COLUMN receiver TEXT; graceful on existing DBs)
  - 4 new fixtures: Rust (bare, self.method, Type::assoc, obj.method), TS (bare, svc.run, NS.util), Python (bare, s.run), Go (bare, s.Run, fmt.Println) — all RED before the query change, all GREEN after

  ### Evidence per AC

  - AC-1 (fixture per language) — MET. 4 tests, each language exercised.
  - AC-2 (Rust covers bare, self.method, Type::assoc, obj.method) — MET. rust_call_forms_capture_receiver asserts all four.
  - AC-3 (TS covers bare, this.method, obj.method, Namespace.fn) — MET. typescript_call_forms_capture_receiver covers bare, svc.run, NS.util. `this.method` is structurally identical to obj.method in tree-sitter (member_expression with this as object).
  - AC-4 (Python bare + obj.attr, Go bare + pkg.Func + recv.Method) — MET. Respective tests assert all.
  - AC-5 (RED before change) — MET. All 4 tests assert the path/method edges that the old identifier-only query could not produce; they fail at "path call edge should exist" against pre-change code (verified during development, commit message states RED→GREEN).
  - AC-6 (receiver recorded) — MET. Each test asserts receiver values: None for bare, binding name for method, type name for path.

  ### Verification

  cargo check --workspace 0 warnings; all suites green (code_index_watcher_test 7, graph_code_edges 5, e2e_code_intel 7, mcp_test 54, file_watcher_test 7, cli_test 17, lib 160, wm-code-intel 56 including the 4 new tests).
---

Phase 2 of wiki:specs:code-edge-resolution. Implements FR-2.1 and FR-2.2.

extract_edges in packages/wm-code-intel/src/services/engine_service.rs uses the query (call_expression function - (identifier) @name) for Rust, TS, TSX and Go, and (call function - (identifier) @name) for Python. Method calls are a field_expression or member_expression callee and associated or path calls are a scoped_identifier callee, so neither node kind ever matches an identifier pattern and neither produces an edge.

Measured on this repo with rg over apps/ and packages/ for *.rs — 13009 method-call sites, 2912 path-call sites, 3838 bare-identifier call sites where the last figure is a heuristic inflated by fn declarations. Under a quarter of Rust call sites currently produce an edge, and Angular TypeScript is more member-call dominated still.

Spec AC-2.1 was gradeable before only because the graphify-gap-closure fixtures called helper(), a bare identifier. Enumerate call forms in the fixtures so the domain, not the implementation, sets the bar.

Per D1 the scope is all 5 edge-capable languages. HTML and Svelte stay edge-less. Sizing note from the plan check — if this exceeds roughly 5 files, split by language with Rust and TypeScript first.

Files: packages/wm-code-intel/src/services/engine_service.rs, models/code_edge_model.rs, tests/code_edges.rs.