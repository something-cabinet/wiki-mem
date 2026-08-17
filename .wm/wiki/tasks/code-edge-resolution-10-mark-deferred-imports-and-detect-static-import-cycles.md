---
title: code-edge-resolution-10 Mark deferred imports and detect static import cycles
type: task
id: "wiki:tasks:code-edge-resolution-10-mark-deferred-imports-and-detect-static-import-cycles"
status: todo
priority: medium
tags: [from-spec, spec:code-edge-resolution, p3, code-intel, cycles]
spec: wiki:specs:code-edge-resolution
acceptance_criteria:
  - text: "A fixture with a static import cycle reports the cycle, and the same cycle formed through a dynamic import does not (spec AC-3.3)"
  - text: "Dynamic imports are marked deferred at extraction (spec FR-3.4)"
  - text: "Import-cycle detection reports cycles over static imports only (spec FR-3.5)"
  - text: "Edge counts for this repo are recorded per phase in the task notes (spec AC-3.4)"
  - text: "Cycle detection is exposed to agents through an existing tool surface rather than a new command"
relates_to:
  - {type: implements, target: wiki:specs:code-edge-resolution}
---

Phase 3 of wiki:specs:code-edge-resolution. Implements FR-3.4, FR-3.5 and NFR-3.1.

Adopted from Graphify per decision D5, and the only analysis feature from Graphify's suite that transfers. Rationale recorded in wiki:reference:graphify-adoption-assessment — Leiden clustering, god-node detection and surprising-connection analysis all produce insight for a human reader; a cycle is a fact that changes what an agent may safely do. The deferred flag is what makes cycle output trustworthy, because a dynamic import does not create a load-order cycle.

Existing cycle handling is wiki-only and informational — apps/wm-core/src/graph/lint.rs notes that mutual relates_to cycles are expected and that BFS uses visited tracking. There is no code-import cycle detection.

Sequenced last because cycles computed over an incomplete import graph would be misleading, so this depends on tasks 04 through 07.

This task also carries the per-phase edge-count recording for the whole of P3 so volume regressions stay attributable.

Files: packages/wm-code-intel/src/services/engine_service.rs, apps/wm-core/src/graph, apps/wm-core/src/mcp/tools/code.rs.