---
title: Core-server graph twin parity P2s and wm_doc write-action output docs
type: task
id: "wiki:tasks:core-server-graph-twin-parity-p2s-and-wmdoc-write-action-output-docs"
status: todo
priority: medium
tags: [from-review, linus-remediation, graph, parity, P2, wm-doc]
acceptance_criteria:
  - text: "Core wm_graph.subgraph builds edges post-BFS from the visited node set (no duplicate emissions, no dangling edge references) matching the wm-server twin"
  - text: "Core and server subgraph apply the same node cap (100) or both drop it, removing the payload-bound divergence"
  - text: "conformance.ts asserts the connect-negotiated protocolVersion equals the server highest-supported once the negotiation fix lands"
  - text: "wm_doc write-action output-shape convergence is either given a documented compat decision or a CHANGELOG entry for the retire-wm-doc path"
---

Non-blocking P2 findings from the Wave 1 review gate (GO-with-findings), captured per findings-first. NOT Wave 1 blockers; P1s already fixed.

T1 P2 (graph twin parity, apps/wm-core/src/mcp/tools/graph.rs vs apps/wm-server/src/routes/graph.rs):
1. Core wm_graph.subgraph builds its edge list inline during BFS from edges_undirected(current), so (a) an edge whose both endpoints are visited is emitted twice, and (b) an edge to a depth+1 neighbor that is never added to nodes becomes a dangling reference. The server twin (routes/graph.rs ~305) builds edges AFTER BFS from graph.edge_indices() filtered to the visited node set. Align core to build edges post-BFS from the node set.
2. Node-cap divergence: server subgraph guards node_ids.len() greater than 100; core wm_graph.subgraph has only a depth guard, no node cap. Align (apply the 100 cap in core, or drop it in server).

T4 P2 (apps/wm-core/src/mcp/tools/doc.rs): wm_doc create/update/delete write-action OUTPUT shapes converged onto the wm_page result (create returns id/path/type, update/delete return id) rather than the legacy status/path/tags. This is intended per retire-wm-doc and now documented in the doc.rs module note; if any external consumer depends on the old shapes, decide whether the retirement path needs a compat shim or a CHANGELOG entry.

T5 P2 (already tracked separately in wiki:tasks:fix-mcp-protocolversion-negotiation): the conformance harness masks the 2024-11-05 fallback; once the negotiation fix lands, add an assert in scripts/mcp-conformance/conformance.ts that the connect-negotiated protocol version equals the server's highest supported.