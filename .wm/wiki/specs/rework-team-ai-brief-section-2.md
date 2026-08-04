---
title: Rework Team AI Brief Section 2 — Capabilities vs Problems
type: spec
status: approved
---

## Overview

The "problem/solution" structure of **Section 2 (wiki-mem — The Memory)** in `eightcap-new-portal/team-ai-setup-brief.md` currently frames every wiki-mem feature as a `### Problem:` block. That framing is wrong for two blocks: **search/scoring** and **code intelligence** are capabilities of the tool, not problems it solves — they belong in introductory capability subsections. This spec reworks Section 2 so genuine problems stay problem-framed, capabilities are introduced as such, and the section reads as: capabilities first, then the problems they solve.

Target: `/Users/nhkhanh/projects/eightcap-new-portal/team-ai-setup-brief.md` — Section 2 only. Sections 1 (Agents) and 3 (Workflow) are unchanged.

## Locked Decisions

- **D1 — Section 2 only**: Restructure only Section 2. Sections 1 (Agents) and 3 (Workflow) keep their current structure and content.
- **D2 — Two capability subsections**: Search/scoring (BM25 + stemming, ONNX semantic, reranking, graph navigation) becomes one "How wiki-mem finds things" capability subsection; code intelligence becomes its own "Code intelligence" capability subsection (AST symbol lookup, reference search, dependency analysis).
- **D3 — Capabilities first, problems after**: New Section 2 order: intro → Local-first data (benefit) → search capability → code-intel capability → the 5 true Problem blocks → metrics disclaimer. Problems come last so they can reference the capabilities.
- **D4 — Trim redundant lines**: The 5 surviving Problem blocks keep their pain + metric attribution but drop inline capability mechanics already covered in the capability subsections (e.g., graph `relates_to` explanation, BM25 internals).

## Requirements

### Functional Requirements

- **FR-1**: Move search/scoring content (keyword search, ONNX semantic, reranking, graph) out of Problem blocks into a capability subsection titled "How wiki-mem finds things" (or equivalent).
- **FR-2**: Move code-intel content (AST, symbol lookup, dependency analysis, zero-LLM retrieval) into its own capability subsection titled "Code intelligence" (or equivalent).
- **FR-3**: Keep exactly 5 true Problem blocks: stateless agents, SDD bloat, fresh sessions, requirements-assumed, timer + critical patterns.
- **FR-4**: Order the section: intro → local-first → search capability → code-intel capability → problems → metrics disclaimer.
- **FR-5**: Trim each surviving Problem block of lines re-explaining capability mechanics already present in the capability subsections; preserve the pain description and any metric attribution.
- **FR-6**: Preserve all existing facts and metrics (50% hours, 80% rework, 50% tokens, zero-token/zero-LLM claims) — the rework is structural, not substantive.

### Non-Functional Requirements

- **NFR-1**: Stays 1–2 pages when rendered — no net length increase from the rework.
- **NFR-2**: Plain language for a non-technical director, consistent with the rest of the brief.
- **NFR-3**: No duplicated content between capability subsections and Problem blocks after the trim.

## Acceptance Criteria

- [ ] AC-1: Section 2 contains two capability subsections (search/scoring, code-intel) and five Problem blocks.
- [ ] AC-2: Capability subsections appear before the Problem blocks.
- [ ] AC-3: No Problem block re-explains search/graph/code-intel mechanics covered by the capability subsections.
- [ ] AC-4: All original facts and metric figures are still present.
- [ ] AC-5: Section 1 and Section 3 are byte-identical to before the rework.
- [ ] AC-6: The brief still renders at 1–2 pages.
- [ ] AC-7: No `### Problem:` block remains for search, semantic, graph, or code-intel topics.

## Scenarios

### Scenario 1: Happy Path — Director reads Section 2
**Given** the reworked Section 2
**When** the director reads it top to bottom
**Then** they first learn what wiki-mem *is* (capabilities: search, graph, code-intel), then the problems it solves — without any Problem block that re-explains the capabilities

### Scenario 2: Agent evaluates retrieval features
**Given** a reviewer wants to verify the zero-token/zero-LLM claims
**When** they read the capability subsections
**Then** the graph and code-intel zero-LLM claims are presented as capability facts, not as problem framings

### Scenario 3: No content loss
**Given** the rework is applied
**When** diffed against the pre-rework brief
**Then** every fact and metric appears somewhere in the new Section 2, and Sections 1/3 are unchanged

## Technical Notes

- Edit the file directly at `/Users/nhkhanh/projects/eightcap-new-portal/team-ai-setup-brief.md` — it is a plain markdown doc, not WM-managed.
- The five Problem blocks to keep: stateless agents (L64), SDD bloat (L79), fresh sessions (L85), requirements-assumed (L88), timer + critical patterns (L91).
- The blocks to convert into capabilities: keyword search + ONNX + graph (L67–78) → "How wiki-mem finds things"; code-intel (L76) → "Code intelligence".
- Local-first data (L79-ish) stays as a benefit subsection, placed after the intro.

## Open Questions

- [ ] Exact wording of the two capability subsection headings ("How wiki-mem finds things" / "Code intelligence" vs. alternatives).
- [ ] Whether the graph deserves its own sub-heading inside the search capability, or stays inline with BM25/ONNX/reranking.
