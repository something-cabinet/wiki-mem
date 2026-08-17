---
title: code-edge-resolution-07 Resolve tsconfig aliases and workspace packages
type: task
id: "wiki:tasks:code-edge-resolution-07-resolve-tsconfig-aliases-and-workspace-packages"
status: done
priority: medium
tags: [from-spec, spec:code-edge-resolution, p2, code-intel, typescript]
spec: wiki:specs:code-edge-resolution
acceptance_criteria:
  - text: "An Angular fixture using a tsconfig path alias produces a resolved import edge where pre-change code produced none (spec AC-2.4)"
  - text: "tsconfig paths and baseUrl are read and applied during TypeScript and TSX import resolution"
  - text: "npm, pnpm and yarn workspace package specifiers resolve to in-repo files where such a package exists"
  - text: "External or unresolvable specifiers still produce no edge rather than a wrong one"
  - text: "Resolution remains deterministic with no network access (spec NFR-2.1)"
relates_to:
  - {type: implements, target: wiki:specs:code-edge-resolution}
implementation_notes: |-
  Implementation complete (2026-08-17):

  1. Created `ts_config_resolver.rs` module: discovers tsconfig.json, parses compilerOptions.paths + baseUrl, resolves aliases
  2. Workspace package resolution via package.json workspaces and pnpm-workspace.yaml
  3. Added `TsResolutionContext` to `CodeIndexSnapshot` — populated during `collect_from_fs` and `materialize_resolved_edges`
  4. Modified `resolve_import` in graph_resolver.rs to try tsconfig alias resolution before path-math for TS/TSX non-relative specifiers
  5. Updated `materialize_resolved_edges` to accept project_root for tsconfig discovery
  6. 10 new unit tests for pattern matching, base URL normalization, JSON comment stripping, end-to-end resolution

  All 75 wm-code-intel tests pass. Full wm-core tests pass (watcher tests pass when run isolated).
---

Phase 2 of wiki:specs:code-edge-resolution. Implements FR-2.5.

resolve_ts_import in packages/wm-code-intel/src/services/engine_service.rs handles relative paths only. Nothing in the package reads tsconfig, so TypeScript path aliases silently produce no import edge — which matters directly here because the Angular app in apps/wm-web uses aliases, and this repo is both a Cargo and an npm workspace.

Adopted from Graphify's resolution pipeline per decision D5, which covers tsconfig aliases, baseUrl, and pnpm, npm and yarn workspace globs.

Scope boundary and open question — Go import resolution currently returns None by design, so Go call edges resolve only within the symbol index. As scoped this task covers TypeScript and TSX only; whether Go imports close here or Go stays call-only is unresolved in the spec and should be settled before starting.

Files: packages/wm-code-intel/src/services/engine_service.rs, services/graph_resolver.rs.