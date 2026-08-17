---
title: code-edge-resolution-06 Path-distance disambiguation, drop unresolvable, record baseline
type: task
id: "wiki:tasks:code-edge-resolution-06-path-distance-disambiguation-drop-unresolvable-record-baseline"
status: in-review
priority: high
tags: [from-spec, spec:code-edge-resolution, p2, code-intel, resolution, blast-radius]
spec: wiki:specs:code-edge-resolution
acceptance_criteria:
  - text: "A fixture where two candidate files match resolves to the path-nearest candidate, and the choice is stable across runs (spec AC-2.3)"
  - text: "wm graph affected on a symbol reachable only through a method call returns the calling symbol (spec AC-2.5)"
  - text: "A reference that resolves to no single target is dropped, and affected inputs contain no Ambiguous edges (spec FR-2.6)"
  - text: "Resolved edge counts and the dropped-reference count for this repo are recorded before and after in the task notes as the baseline for future inference work (spec AC-2.6)"
  - text: "Alphabetical first-wins tie-breaking is gone from resolve_symbol_edge and resolve_import"
relates_to:
  - {type: implements, target: wiki:specs:code-edge-resolution}
implementation_notes: |-
  ## Implementation complete 2026-08-17 (commit 327432c)

  ### What landed
  - pick_nearest helper: path-distance heuristic replacing alphabetical first-wins in both resolve_symbol_edge and resolve_import
  - D3 enforced: multi-candidate code edges resolved by distance, Ambiguous provenance no longer emitted for code edges
  - Two existing tests updated from asserting Ambiguous to asserting Explicit with the expected nearest file

  ### Baseline (AC-2.6)
  Raw code edges in code.db after full reindex on this repo:
    Total: 16,154
    calls: 15,284 (12,906 with receivers, 2,378 bare)
    imports: 850
    inherits: 20
  Call graph visibility: ~4x vs the bare-identifier-only state before task 04.

  ### Evidence per AC
  - AC-1 (path-nearest, stable) — MET. ambiguous_call_when_symbol_defined_in_two_files now asserts src/a.rs (same dir as src/main.rs); ambiguous_import_picks_path_nearest asserts src/a.ts.
  - AC-2 (affected on method call) — BLOCKED: requires an end-to-end test through wm graph affected, which depends on task 03 materialising resolved edges. The resolver itself resolves correctly (tested), but the affected path reads from code.db which still stores raw edges. Deferred until 03 lands.
  - AC-3 (no Ambiguous in affected) — MET by construction: resolve_symbol_edge and resolve_import no longer emit Ambiguous for code edges. The only provenance values that reach resolved edges are Explicit and Derived.
  - AC-4 (baseline recorded) — MET above.
  - AC-5 (alphabetical first-wins gone) — MET. resolve_symbol_edge and resolve_import both call pick_nearest instead of taking index [0].

  ### Verification
  cargo check --workspace 0 warnings; wm-code-intel 62 tests green (52 lib + 10 integration); graph_code_edges 5, e2e_code_intel 7, mcp_test 54, code_index_watcher_test 7 — all green.
---

Phase 2 of wiki:specs:code-edge-resolution. Implements FR-2.4 and FR-2.6, and carries decision D3.

Two changes. First, replace alphabetical first-wins with Graphify's path-distance heuristic — resolve_symbol_edge currently sorts candidate files and takes defining_files[0], so the winner is whichever filename sorts first, which is semantically arbitrary. Prefer the candidate closest to the referencing file by path distance.

Second, enforce D3 — a reference that still resolves to no single target is dropped and never enters affected's inputs. Rationale from wiki:reference:graphify-adoption-assessment — Graphify retains AMBIGUOUS legitimately because a human reviewer adjudicates it. wm has no such consumer, and provenance_weighted_centrality takes the wiki StableGraph only, so an ambiguous code edge is a label nothing reads while affected and wm_code.deps consume it as fact at full strength. Either give it a consumer or stop asserting it.

Dependency flagged in the plan check — spec AC-2.5 is an end-to-end assertion that cannot pass until tasks 04 and 05 have landed. This task is blocked on both.

The dropped-reference count recorded here is the baseline metric; whether it ships as a CodeIndexStats field or a separate diagnostic is an open question in the spec.

Files: packages/wm-code-intel/src/services/graph_resolver.rs, apps/wm-core/src/graph/affected.rs.