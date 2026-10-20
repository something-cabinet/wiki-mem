---
title: wm-core exceeds its 13,000-line Gate 1 ceiling after the code-index work
type: task
id: "wiki:tasks:wm-core-exceeds-its-13000-line-gate-1-ceiling-after-the-code-index-work"
status: in-progress
priority: high
tags: [ci, wm-core, architecture, structure, gate-1]
acceptance_criteria:
  - text: "Gate 1 passes at HEAD without weakening the wm-core ceiling"
  - text: "No behavior change to code-index refresh, graph loading, or CLI commands"
  - text: "All wm-core, wm-tools, and wm-cli tests stay green"
  - text: "CI `check` job is green end to end"
relates_to:
  - {type: implements, target: wiki:specs:wm-core-code-intel-service-move}
---

CI run 39397172947 (commit 394017c) now reaches Gate 1 and fails: `apps/wm-core: 13271 lines > ceiling 13000 and no waiver in .github/workspace-crates.toml`. The violation predates the code-index commits — `28377ed` was already 13,053 — but this session added 184 lines (13,053 -> 13,271), and the earlier file-size failure had masked Gate 1.

Options: (1) move the code-intel-specific modules out of apps/wm-core into packages/wm-code-intel (`engine/code_index_refresh_service.rs` 243 lines and `graph/code_edges.rs` 73 lines; result ~12,961 with re-export stubs), or (2) add a temporary `[waivers.wm-core]` reason and a removal task. The ceiling is deliberate (spec wm-core-layer-extraction D-11) and meant to prevent wm-core regrowth.