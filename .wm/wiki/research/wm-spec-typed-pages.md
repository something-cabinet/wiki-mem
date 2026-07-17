---
title: Leveraging WM Typed Pages + Edges in wm-spec
type: concept
---

## Research: wm-spec × WM Typed Pages + Edges

### Current state
wm-spec creates a single flat `wiki/specs/<name>` doc. Same as Knowns. Ignores WM's 9 page types and 16 edge types entirely.

### Core insight
The SDD (Socratic Spec-Driven Development) process produces structured intermediate artifacts — locked decisions, functional requirements, open questions, domain concepts, scenarios — but wm-spec **discards all structure** by flattening them into prose markdown. WM's typed pages + edges can preserve and connect this structure.

### Mapping

| SDD element | WM page type | Per-type data | Edge to spec |
|---|---|---|---|
| Locked decision | Decision | context, options, rationale, outcome | `answers` |
| Open question | Decision (draft) | context, options | `questions` |
| Functional requirement | Task | task_data.acceptance_criteria | `implements` |
| Acceptance criterion | (embedded in TaskData) | — | — |
| Domain concept | Concept | — | `extends` |
| Scenario | Concept/Howto | — | `example_of` |
| Technical note | Note | — | `references` |
| Supporting reference | Reference | — | `references` |
| Task dependency | — | — | Task `depends_on`→Task |
| Superseded decision | — | — | Decision `supersedes`→Decision |

### Proposed workflow

**Phase 1 — During Socratic Exploration (Step 0):**
For each locked decision, immediately create a Decision page with ADR frontmatter and link via `answers` edge. Open questions become Decision (draft) pages with `questions` edge. Concept pages optionally created for domain concepts.

**Phase 2 — After spec creation (Step 3):**
Auto-generate Task pages from FRs with `implements` edge to the spec. Each task carries acceptance criteria derived from the spec's ACs. This replaces the manual `/wm-plan` step entirely.

**Phase 3 — After approval (Step 5):**
Update all generated pages' statuses atomically (Decision pages → `approved`, Task pages → `todo`).

### Open questions
1. Should task auto-generation from FRs be automatic or optional?
2. Should open questions become Decision pages automatically or stay inline?
3. How to handle backward compat with existing flat specs?

### Full doc
See `tmp/research-wm-spec-typed-pages.md` for detailed analysis.