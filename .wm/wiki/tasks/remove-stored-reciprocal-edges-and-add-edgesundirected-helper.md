---
title: Remove stored reciprocal edges and add edges_undirected helper
type: task
id: wiki:tasks:remove-stored-reciprocal-edges-and-add-edgesundirected-helper
status: done
priority: high
tags:
- from-oracle
- refactor
- simplification
- linus-remediation
parent: wiki:tasks:apply-oracle-recommendations-from-linus-critique-review
acceptance_criteria:
- text: Reciprocal edge storage deleted (graph/mod.rs:150-163 pass + reconciliation set)
- text: edges_undirected helper serves all 5 consumer sites (query.rs:67, affected.rs:75, export.rs:66, graph tools, UI)
- text: Phantom-source bug (graph/mod.rs:186-199) eliminated
- text: Double-counted degree in exports fixed
- text: cargo build + clippy -D warnings + graph/search suites green
implementation_notes: 'Wave 1 review gate: GO-with-findings. P1 (find_path was Outgoing-only) FIXED — now uses crate::graph::edges_undirected + non-current endpoint selection; added regression test graph::tests::find_path_traverses_reverse_direction_without_stored_reciprocals (reverse b->a reachability). P2 degree self-loop double-count FIXED at mcp/tools/graph.rs to edges_undirected(graph,idx).len() (matches wm-server twin). Remaining P2s (core subgraph edge-set built inline vs post-BFS, node-cap divergence) moved to wiki:tasks:core-server-graph-twin-parity-p2s-and-wmdoc-write-action-output-docs. Verified: clippy -D warnings clean; graph lib tests 19; mcp_test 54; e2e_graph 3; graph_code_edges 5. All ACs satisfied.'
---

From wiki:tasks:apply-oracle-recommendations-from-linus-critique-review AC-1. Oracle verdict LANDED: reciprocal backlink edges stored at graph/mod.rs:150-163 tagged Derived, but reverse view is one iterator call away (query.rs:67-70, affected.rs:75, export.rs:66-68 already use edges_directed Incoming). Stored transpose drags 4 compensating artifacts: double-counted degree in exports, 0.5 ranking weight, UI legend tier, phantom-source bug (graph/mod.rs:186-199). Also only covers body @wiki refs not frontmatter relates_to (incoherent semantics). Fix: delete the reciprocal pass + reconciliation set; add one edges_undirected(graph, idx) helper (Outgoing+Incoming); keep Derived enum variant for genuinely auto-created edges (P2 code-intel re-export resolution).