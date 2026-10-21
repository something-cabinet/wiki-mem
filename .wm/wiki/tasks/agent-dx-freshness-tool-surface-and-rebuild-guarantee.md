---
title: Agent DX Tier 1 — index freshness in tool results, tool surface budget, wm doctor + rebuild guarantee
type: task
id: "wiki:tasks:agent-dx-freshness-tool-surface-and-rebuild-guarantee"
status: todo
tags: [agent-dx, freshness, mcp, tool-surface, index-integrity]
acceptance_criteria:
  - {text: "Every read tool listed in spec FR-1 returns indexed_at, stale_count, and stale_files, with a warning only when stale", checked: false}
  - {text: "Read tools do not mutate index state", checked: false}
  - {text: "The MCP tool-surface budget is enforced by a test or gate", checked: false}
  - {text: "wm doctor reports index health and offers a rebuild guarantee, with tests for a stale and a missing index", checked: false}
---

schema_version: 1
state: |-

  Severity: Medium

  Implement Tier 1 of @wiki/specs/agent-dx-freshness-and-tool-surface Three clusters, one task umbrella — split into per-cluster tasks when planning from the spec.

  ## Acceptance Criteria

  - [ ] Index freshness (`indexed_at`, `stale_count`, `stale_files`) is present on every read tool listed in spec FR-1, with a warning only when stale
  - [ ] Read tools do not mutate index state (verified by before/after store hash)
  - [ ] Unknown freshness reports `null` plus an "unknown" warning, never a false "fresh"
  - [ ] `wm doctor` reports disk↔derived drift with `file:line`, exits non-zero on drift, writes nothing, and does not duplicate lint/validate checks
  - [ ] CI test: two consecutive rebuild-from-pages runs produce identical graph edges and BM25 results for a fixed corpus; a graph-producing edit without rebuild fails the test
  - [ ] Registry test asserts all four behavior annotations on every registered tool
  - [ ] `mcp.profile = lean` advertises ≤12 curated tools + `wm_context` + `wm_help`; `full` is unchanged; measured token numbers recorded in the spec
  - [ ] `core/ARCHITECTURE` states the derived-index invariant (spec FR-15)

  ## Files (expected touch points)

  - `apps/wm-core/src/mcp/tools/*` — freshness assembly + annotations + profile filter
  - `apps/wm-core/src/mcp/tools/index.rs` — reuse existing code-index age computation
  - `packages/wm-engine` — tool registry choke point for annotations and profile
  - `packages/wm-code-intel` — code-index staleness source
  - `apps/wm-cli` — `wm doctor` command + profile config
  - `.wm/wiki/core/ARCHITECTURE.md` — derived-index invariant
  - CI workflow — rebuild-equivalence test

  ## Blocked On

  Open Questions in the spec: `wm_context` vs the existing context-assembly path (DRY), and the frozen lean-core tool list (needs the token measurement first).

  Spec: @wiki/specs/agent-dx-freshness-and-tool-surface
Research: @wiki/reference/peer-landscape-2026