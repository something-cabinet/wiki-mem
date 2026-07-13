---
title: Canonical Steering Alignment — WIKI-MEM.md + Compat Shims
description: Create WIKI-MEM.md as canonical source, subsume .wm/AGENTS.md, update all compat shims to Knowns steering pattern
createdAt: '2026-07-09T17:24:06.189Z'
updatedAt: '2026-07-09T17:27:26.325Z'
tags:
  - spec
  - approved
  - steering
  - shims
---

## Overview

Align vpp-rag's agent steering files with Knowns' proven pattern. Create `WIKI-MEM.md` as the single canonical source of truth (subsuming `.wm/AGENTS.md`), update all platform compat shims to delegate to it with inline Minimum Rules and GUIDELINES markers, and ensure every platform auto-detect entrypoint exists.

## Locked Decisions

- **D-1:** `WIKI-MEM.md` is the canonical source, shims delegate to it.
- **D-2:** `WIKI-MEM.md` subsumes `.wm/AGENTS.md` — merge workflows into it, deprecate `.wm/AGENTS.md`.
- **D-3:** Create both `OPENCODE.md` and `.github/copilot-instructions.md` as new compat shims.

## Requirements

### Functional

- FR-1: WIKI-MEM.md exists at repo root as the single canonical source of truth for agent behavior
- FR-2: WIKI-MEM.md subsumes all content from `.wm/AGENTS.md` (wiki conventions, 8 canonical workflows, tool usage rules)
- FR-3: WIKI-MEM.md follows KNOWNS.md structure (Source of Truth, TL;DR, Tool Selection, Memory Usage, Critical Rules, Git Safety, Context Retrieval, Common Mistakes, etc.) but is WM-native (references `wm_*` tools, `knowns_*` MCP tools, project-specific conventions)
- FR-4: Every compat shim (AGENTS.md, CLAUDE.md, GEMINI.md, OPENCODE.md, .github/copilot-instructions.md) follows this template:
  - `<!-- WIKI-MEM GUIDELINES START -->` marker
  - CRITICAL: delegate to WIKI-MEM.md as canonical
  - Canonical Guidance section (1-2 paragraphs)
  - Minimum Rules section (inline, standalone useful without WIKI-MEM.md)
  - Quick Reference with WM CLI commands
  - `<!-- WIKI-MEM GUIDELINES END -->` marker
- FR-5: `.wm/AGENTS.md` is deprecated — content merged into WIKI-MEM.md, file replaced with a compat shim pointing to WIKI-MEM.md (or removed)
- FR-6: `KNOWNS.md` explicitly states WIKI-MEM.md is canonical for operational guidance (or delegates to it)
- FR-7: `OPENCODE.md` created for OpenCode auto-detection (OpenCode detects both AGENTS.md and OPENCODE.md)
- FR-8: `.github/copilot-instructions.md` created for GitHub Copilot auto-detection

### Non-Functional

- NFR-1: WIKI-MEM.md uses `wm_*` tool prefix throughout, not `knowns_*`
- NFR-2: Compat shims stay lightweight (under ~60 lines each)
- NFR-3: Minimum Rules in each shim must be sufficient for an agent to operate correctly without having read WIKI-MEM.md yet
- NFR-4: WIKI-MEM.md ready-only — processed by humans and agents alike

## Acceptance Criteria

- AC-1: WIKI-MEM.md exists at repo root, subsuming `.wm/AGENTS.md` content with WM-native KNOWNS.md structure
- AC-2: AGENTS.md updated: GUIDELINES markers + Minimum Rules + delegates to WIKI-MEM.md
- AC-3: CLAUDE.md updated: same pattern as AC-2
- AC-4: GEMINI.md updated: same pattern as AC-2
- AC-5: OPENCODE.md created: same pattern as AC-2
- AC-6: .github/copilot-instructions.md created: same pattern as AC-2
- AC-7: .wm/AGENTS.md deprecated (replaced with shim or removed; content in WIKI-MEM.md)
- AC-8: (removed — KNOWNS.md deleted)
- AC-9: All files have consistent language, tool names, and conventions
- AC-10: Quick Reference in each shim lists correct WM CLI commands (wm-cli serve/search/page/lint)

## Scenarios

### Happy Path

1. New agent session starts in vpp-rag repo
2. Runtime auto-detects AGENTS.md (or OPENCODE.md, CLAUDE.md, etc.)
3. Agent reads shim → CRITICAL: delegates to WIKI-MEM.md
4. Minimum Rules give agent enough context to proceed immediately
5. Agent reads WIKI-MEM.md for full steering
6. Agent follows workflow rules, uses `wm_*` tools, captures memory

### Edge Cases

- **Agent runtime that only reads OPENCODE.md** — OPENCODE.md exists, has Minimum Rules, delegates to WIKI-MEM.md
- **Agent runtime that only reads CLAUDE.md** — same coverage
- **Agent runtime that never reads WIKI-MEM.md** — Minimum Rules in the shim provide enough standalone guidance
- **Agent runs before WIKI-MEM.md is created** — compat shims currently work; migration window is safe
- **Cross-project agent with knowns_* muscle memory** — WIKI-MEM.md maps knowns_* to wm_* equivalents explicitly

## Open Questions

(none — fully scoped)

