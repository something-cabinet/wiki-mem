---
title: C1 Remove web-only graph remnants
type: task
id: "wiki:tasks:c1-remove-web-only-graph-remnants"
status: todo
priority: medium
tags: [cleanup, graph, web-remnant]
acceptance_criteria:
  - text: "wm_graph.full removed (action, handler, registration, CLI refs)"
  - text: "Tests + agent instructions no longer reference wm_graph.full"
  - text: "Stale 'HTTP graph routes' doc line fixed"
  - text: "Dead cached_db_is_fresh stub removed or justified"
  - text: "cargo check/clippy/tests clean; grep-clean"
---

Remove web-only graph remnants after wm-web retirement: the `wm_graph.full` tool (viz contract), stale doc line, and the dead `cached_db_is_fresh` no-op. Per the graph-usage recon.