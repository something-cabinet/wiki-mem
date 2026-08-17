---
title: code-edge-resolution-08 Split implements from inherits
type: task
id: "wiki:tasks:code-edge-resolution-08-split-implements-from-inherits"
status: todo
priority: medium
tags: [from-spec, spec:code-edge-resolution, p3, code-intel, edge-types]
spec: wiki:specs:code-edge-resolution
acceptance_criteria:
  - text: "A Rust fixture with impl Display for Foo yields an implements edge, and a fixture with trait A - B yields inherits (spec AC-3.1)"
  - text: "TypeScript implements clauses yield implements, and TypeScript extends yields inherits"
  - text: "Existing inherits edges for Rust impl-trait are migrated to implements without leaving duplicates"
  - text: "affected still treats both edge types as break-sensitive"
  - text: "The edge type is registered wherever code edge types are enumerated so no unknown-type warning appears"
relates_to:
  - {type: implements, target: wiki:specs:code-edge-resolution}
---

Phase 3 of wiki:specs:code-edge-resolution. Implements FR-3.1.

extract_edges currently folds two distinct relations into inherits. The Rust query (impl_item trait - (type_identifier) @base type - (type_identifier) @name) captures impl Trait for Type, which is semantically implements, not inheritance. TypeScript class_heritage extends is genuine inheritance, and TypeScript implements clauses are not captured at all.

Correct mapping per the spec — Rust impl Trait for T is implements, Rust supertraits and TypeScript extends are inherits, TypeScript implements clauses are implements.

Adopted from Graphify's 12-relation taxonomy per decision D5. Cheapest of the P3 items and the one that makes trait-implementation queries answerable, which matters in a Rust codebase where trait impls carry much of the behaviour.

Note from wiki:reference:graphify-adoption-assessment — richer relations have higher return in wm than in Graphify, because wm has affected, ranking and search as consumers where Graphify has visualizations.

Files: packages/wm-code-intel/src/services/engine_service.rs, models/code_edge_model.rs, apps/wm-core/src/graph/affected.rs.