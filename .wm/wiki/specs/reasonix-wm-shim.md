---
title: Add REASONIX.md to wm init
type: spec
status: draft
---

—-
title: Add REASONIX.md to wm init
type: spec
status: draft
tags: [reasonix, wm, init, shim]
—-

## Overview

Add `REASONIX.md` to the list of compatibility shims that `wm init` generates, alongside the existing CLAUDE.md, GEMINI.md, OPENCODE.md, and AGENTS.md. REASONIX.md follows the exact same template: WIKI-MEM.md redirect wrapped in `<!— WIKI-MEM GUIDELINES —>` markers.

This is purely a WM change. The orchestrator (skills, ORCHESTRATOR.md, global config) is handled by the separate `reasonix-orchestrate` binary.

## Requirements

### Functional Requirements

- FR-1: `wm init` wizard includes Reasonix as a platform option in the platform selection prompt (alongside claude, opencode, kiro, gemini, copilot)
- FR-2: `sync_agent_files()` generates `REASONIX.md` when Reasonix is selected, following the OPENCODE.md pattern exactly
- FR-3: `REASONIX.md` content:
  - `# REASONIX` header, "Compatibility entrypoint" line
  - `<!— WIKI-MEM GUIDELINES START —>` / `<!— WIKI-MEM GUIDELINES END —>` markers
  - CRITICAL directive pointing to WIKI-MEM.md as canonical
  - Canonical Guidance section (same 4 bullets as OPENCODE.md)
  - Quick Reference section with wm-cli commands
- FR-4: `wm init —no-wizard —platform reasonix` generates REASONIX.md only (headless)
- FR-5: `wm setup reasonix` generates REASONIX.md (mirrors `wm setup opencode`)

### Non-Functional Requirements

- NFR-1: Zero change to existing shim files (CLAUDE.md, GEMINI.md, etc.)
- NFR-2: No new dependencies

## Acceptance Criteria

- [ ] AC-1: `wm init` with Reasonix selected creates REASONIX.md matching the OPENCODE.md pattern
- [ ] AC-2: `wm init —no-wizard —platform reasonix` creates REASONIX.md without prompts
- [ ] AC-3: `wm setup reasonix` generates REASONIX.md
- [ ] AC-4: Existing shims are unchanged
- [ ] AC-5: No .reasonix/ directory or skills are created by WM — that's the orchestrator's job

## Scenarios

### Scenario 1: Interactive init with Reasonix
**Given** a user runs `wm init` interactively and selects "reasonix" in the platform prompt
**When** the command completes
**Then** REASONIX.md exists alongside CLAUDE.md, GEMINI.md, etc.

### Scenario 2: Headless
**Given** a user runs `wm init —no-wizard —platform reasonix`
**When** the command completes
**Then** REASONIX.md is the only generated shim

## Technical Notes

- `sync_agent_files()` in `apps/wm-cli/src/main.rs` — add `"reasonix"` to the platform list and generate REASONIX.md with the same template pattern as OPENCODE.md
- The platform selection prompt in the init wizard already accepts comma-separated numbers — `reasonix` is just another number in the list
- `wm setup reasonix` already works if `sync_agent_files()` handles the platform name — no separate handler needed