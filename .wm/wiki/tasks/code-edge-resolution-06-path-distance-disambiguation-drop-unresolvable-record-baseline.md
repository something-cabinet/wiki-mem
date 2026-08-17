---
title: code-edge-resolution-06 Path-distance disambiguation, drop unresolvable, record baseline
type: task
id: "wiki:tasks:code-edge-resolution-06-path-distance-disambiguation-drop-unresolvable-record-baseline"
status: todo
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
---

Phase 2 of wiki:specs:code-edge-resolution. Implements FR-2.4 and FR-2.6, and carries decision D3.

Two changes. First, replace alphabetical first-wins with Graphify's path-distance heuristic — resolve_symbol_edge currently sorts candidate files and takes defining_files[0], so the winner is whichever filename sorts first, which is semantically arbitrary. Prefer the candidate closest to the referencing file by path distance.

Second, enforce D3 — a reference that still resolves to no single target is dropped and never enters affected's inputs. Rationale from wiki:reference:graphify-adoption-assessment — Graphify retains AMBIGUOUS legitimately because a human reviewer adjudicates it. wm has no such consumer, and provenance_weighted_centrality takes the wiki StableGraph only, so an ambiguous code edge is a label nothing reads while affected and wm_code.deps consume it as fact at full strength. Either give it a consumer or stop asserting it.

Dependency flagged in the plan check — spec AC-2.5 is an end-to-end assertion that cannot pass until tasks 04 and 05 have landed. This task is blocked on both.

The dropped-reference count recorded here is the baseline metric; whether it ships as a CodeIndexStats field or a separate diagnostic is an open question in the spec.

Files: packages/wm-code-intel/src/services/graph_resolver.rs, apps/wm-core/src/graph/affected.rs.