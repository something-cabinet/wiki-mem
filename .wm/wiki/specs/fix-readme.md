---
title: Fix README
type: spec
id: wiki:specs:fix-readme
status: approved
tags:
- docs
- readme
- accuracy
- cli-naming
---

## Overview

Correct inaccuracies in the repository's user-facing documentation and remove a stale root-level architecture document.

- Both `README.md` (root) and `docs/README.md` invoke the CLI as `wm <command>` — the actual binary is `wm-cli` (verified on PATH; `wm` is not installed).
- The Install section references a placeholder npm scope (`@something-cabinet/wm-cli`).
- The root README contains an "Or build from source: `cargo install wm-cli`" block that must be removed.
- `ARCHITECTURE-SPEC.md` is a stale root-level document describing an obsolete phase-gated migration model; it must be deleted and its wiki references cleaned up.

This is an accuracy pass: fix wrong statements, do not restructure sections, rewrite prose, or change layout.

## Locked Decisions

- D1: Root `README.md` + `docs/README.md` in scope. Other READMEs (`apps/wm-web/`, `scripts/docker-stress/`) and wiki page `wiki:core:README` are out of scope.
- D2: Accuracy pass only — correct inaccuracies; no restructure, tone, or layout changes.
- D3: Code/runtime ground truth — verify claims against `wm-cli --help`, `wm-cli setup --help`, the ToolRegistry surface, and repo source. Unverifiable items become Open Questions, never guesses.
- D4: All command invocations use `wm-cli` — never `wm`. Verified: `wm-cli` on PATH (npm install), `wm` not found.
- D5: Delete `ARCHITECTURE-SPEC.md` AND fix wiki pages referencing it (1 live `@doc` ref in `decisions/wm-server-overrides-tauri-primary.md`; prose mentions in tasks `d93671`, `ef4616`, `2a335e`) so `wm_validate` stays clean.
- D6: Remove the "Or build from source: `cargo install wm-cli`" block from root `README.md`.

## Requirements

### Functional Requirements

- FR-1: Every CLI command invocation in both READMEs uses the correct binary name `wm-cli` (e.g., `wm-cli init`, `wm-cli mcp`, `wm-cli search`).
- FR-2: The Install section of root `README.md` no longer contains the "Or build from source: `cargo install wm-cli`" block; remaining install instructions use the correct published package name.
- FR-3: The CLI Commands table in both READMEs matches the actual command surface from `wm-cli --help` — no stale, missing, or renamed commands.
- FR-4: The `wm-cli setup <platform>` platform list and examples match actual `wm-cli setup` targets.
- FR-5: The MCP Tools table in both READMEs matches the current ToolRegistry surface (names, counts, groups).
- FR-6: Architecture (root README) and Web UI (docs/README) sections accurately reflect the current deployment model (single binary serving the Angular frontend on :4090) — verified against source.
- FR-7: The `.mcp.json` example in root README uses the correct command (`wm-cli mcp`).
- FR-8: Factual claims in docs/README (page ranks, edge types, MCP tool counts, skill count, search parameters) checked against source and corrected if wrong.
- FR-9: Markdown integrity — no broken code blocks, tables, or internal links introduced.
- FR-10: `ARCHITECTURE-SPEC.md` deleted from repo root.
- FR-11: Wiki pages referencing `ARCHITECTURE-SPEC.md` updated: remove/replace the `@doc` reference in `decisions/wm-server-overrides-tauri-primary.md`; update prose mentions in tasks `d93671`, `ef4616`, `2a335e`.
- FR-12: `wm_validate.check` passes after ref cleanup (no broken refs introduced by the deletion).

### Non-Functional Requirements

- NFR-1: Changes are edits to existing content only — section ordering, headings, and overall layout unchanged.
- NFR-2: No scope creep beyond the two README files, `ARCHITECTURE-SPEC.md` deletion, and its wiki reference cleanup.

## Acceptance Criteria

- [ ] AC-1: Zero bare `wm <command>` invocations remain in either README (grep for `wm (init|mcp|search|web|setup|page|graph|task|time|model|index|lint|validate)` returns nothing except prose references to the project name).
- [ ] AC-2: Root README Install section has no `cargo install wm-cli` / build-from-source block; remaining install instructions accurate.
- [ ] AC-3: CLI Commands tables match `wm-cli --help` output (verified during implementation; diff documented).
- [ ] AC-4: Platform setup list matches `wm-cli setup --help` targets.
- [ ] AC-5: MCP Tools tables match current ToolRegistry (counts and group names verified against source).
- [ ] AC-6: Architecture/Web UI sections reflect actual deployment (verified against source).
- [ ] AC-7: `.mcp.json` example uses `wm-cli mcp`.
- [ ] AC-8: Both READMEs render as valid markdown (no broken tables/code blocks).
- [ ] AC-9: No layout/restructure changes — diff limited to content corrections.
- [ ] AC-10: `ARCHITECTURE-SPEC.md` no longer exists in the repo.
- [ ] AC-11: All 4 wiki pages referencing `ARCHITECTURE-SPEC.md` updated; `wm_validate.check` reports no broken refs from this change.

## Scenarios

### Scenario 1: Happy Path
**Given** a new user reads the root README
**When** they follow Install + Quick Start
**Then** commands they run (`wm-cli init`, `wm-cli mcp`, etc.) match the real binary, and install uses a real package name

### Scenario 2: CLI Surface Drift
**Given** the CLI has commands not listed in the README (or listed commands that were renamed)
**When** the table is compared to `wm-cli --help`
**Then** the table is corrected to match reality; unverifiable entries flagged as Open Questions

### Scenario 3: Placeholder Artifact
**Given** the npm scope `@something-cabinet` is a placeholder
**When** the published package name cannot be confirmed at implementation time
**Then** install section is corrected with only accurate instructions, and the real package name is recorded as an Open Question

### Scenario 4: Stale Architecture Doc
**Given** `ARCHITECTURE-SPEC.md` describes an obsolete deployment model and wiki pages link to it
**When** the file is deleted
**Then** all wiki references are updated, and `wm_validate.check` passes with no new broken refs

## Technical Notes

- Verification commands during implementation: `wm-cli --help`, `wm-cli setup --help`, `wm-cli web --help`.
- ToolRegistry surface: check `apps/wm-core/src/tool_registry.rs` (or equivalent) for authoritative tool group/count listing.
- The `@doc/specs/ARCHITECTURE-SPEC.md` reference in the decision page must be removed or retargeted to the actual architecture doc (`@wiki/core:architecture` or `docs/ARCHITECTURE.md` if present).
- docs/README.md contains the same "Or build from source" block as root README — see Open Question Q1.

## Open Questions

- [ ] Q1: Should the identical "Or build from source: `cargo install wm-cli`" block in `docs/README.md` also be removed? (User explicitly requested root README only.)
- [ ] Q2: What is the real published npm package name for `wm-cli`? If it cannot be confirmed at implementation time, keep only verified install instructions.