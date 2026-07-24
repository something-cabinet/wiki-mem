---
id: wiki:specs:wm-spec-typed-pages
title: wm-spec Typed Pages + Edges
type: spec
status: approved
tags: [spec, wm-spec, typed-pages, edges]
relates_to:
  - {type: references, target: wiki:tasks:research-leverage-wm-typed-pages--edge-relations-in-wm-spec}
---
id: wiki:specs:wm-spec-typed-pages
## Overview

Evolve `wm-spec` from creating a single flat `wiki/specs/<name>` doc to creating a **connected graph of typed pages**: Decision pages for locked decisions and open questions, Task pages for functional requirements, Concept pages for domain concepts — all linked via typed edges to the spec anchor page.

This eliminates the manual `/wm-plan` step (tasks are ready for `/wm-flow` immediately after spec approval) and makes every artifact searchable, editable, and traversable independently.

## Locked Decisions

- D1: Task auto-generation from FRs is **automatic**. Every FR generates a Task page with `implements` edge.
- D2: Open questions → **Decision (draft)** pages with `questions` edge.
- D3: Domain concepts → **Concept** pages with `extends` edges (auto-create).
- D4: Existing specs → **one-time migration** pass to create Decision/Task pages.
- D5: Spec page stays as **lightweight anchor** with edges to generated pages.

## Requirements

### Functional Requirements

- FR-1: During Socratic Exploration (Step 0), each locked decision creates a **Decision page** at `wiki:decisions/<spec-name>-<slug>` with full ADR frontmatter (context, options, rationale, outcome) and a `answers` edge to the spec.
- FR-2: Each open question from exploration creates a **Decision (draft)** page at `wiki:decisions/<spec-name>-<slug>` with `questions` edge to the spec.
- FR-3: After spec creation (Step 3), each Functional Requirement (FR-1, FR-2, ...) auto-generates a **Task page** at `wiki:tasks/<spec-name>-fr-<n>` with:
  - Title: `FR-<n>: <description>`
  - Acceptance criteria derived from the spec's matching ACs
  - `implements` edge → spec
  - `depends_on` edges → prerequisite tasks (if FRs reference each other)
- FR-4: Domain concepts named during exploration auto-create **Concept pages** at `wiki:concepts/<slug>` with `extends` edges to related concepts and a `references` edge to the spec.
- FR-5: The spec page remains as the anchor. Its `relates_to` list includes all generated pages.
- FR-6: After spec approval (Step 5), all generated pages sync their statuses:
  - Decision (locked) → `approved`
  - Decision (open) → `draft` (unchanged — user resolves later)
  - Task pages → `todo`
  - Concept pages → `reviewed`
- FR-7: Existing specs (pre-enhancement) get a one-time migration pass. The skill reads their Locked Decisions section, creates Decision pages, reads FRs, creates Task pages, links them.
- FR-8: The wm-spec SKILL.md is updated to document the new workflow, page type mapping, and edge conventions.

### Non-Functional Requirements

- NFR-1: The spec doc template is updated to be lighter — removes inline Locked Decisions and FRs (they live in linked pages). Spec body focuses on scenarios, constraints, and technical notes.
- NFR-2: Existing spec docs are not deleted or modified by the new flow — only by the optional migration pass.
- NFR-3: Page creation and edge linking is atomic — if any page creation fails, the spec is still created as a flat doc (graceful degradation).
- NFR-4: No breaking changes to `/wm-flow` or `/wm-plan` — they should still accept both flat specs and page-graph specs.

## Acceptance Criteria

- [ ] AC-1: Running `wm-spec` with a new feature creates Decision pages for each locked decision, linked via `answers` edge.
- [ ] AC-2: Open questions from Step 0 become Decision (draft) pages with `questions` edge.
- [ ] AC-3: Each FR-* in the spec generates a Task page with `implements` edge to the spec.
- [ ] AC-4: Domain concepts from exploration auto-create Concept pages with `extends` edges.
- [ ] AC-5: The spec page's `relates_to` lists all generated pages.
- [ ] AC-6: After spec approval, all generated pages get correct statuses (Decision→approved, Task→todo, Concept→reviewed).
- [ ] AC-7: Migration pass converts an existing flat spec into the page graph format.
- [ ] AC-8: `cargo test` passes with same count.
- [ ] AC-9: `/wm-flow @doc/specs/<name>` works on the new graph-format spec (reads the anchor page, discovers linked tasks via edges).

## Scenarios

### Scenario 1: New spec with 3 FRs and 2 decisions
**Given** a user specs "user-auth" with 3 FRs and 2 locked decisions
**When** wm-spec completes
**Then** there are 6 new pages:
- 1 spec anchor (`wiki:specs/user-auth`)
- 2 decision pages (`wiki:decisions/user-auth-jwt`, `wiki:decisions/user-auth-session`), each with `answers`→spec
- 3 task pages (`wiki:tasks/user-auth-fr-1`..3), each with `implements`→spec
- Plus any concepts discovered
**And** `/wm-flow @doc/specs/user-auth` discovers 3 tasks via graph traversal

### Scenario 2: Migration of existing flat spec
**Given** `wiki:specs/existing-feature` has inline Locked Decisions and FRs in markdown
**When** migration pass runs
**Then** it parses the markdown, creates Decision pages with `answers`→spec, creates Task pages with `implements`→spec, updates the spec's frontmatter `relates_to`

### Scenario 3: Graceful degradation on page creation failure
**Given** creating a Decision page fails (disk full, invalid slug)
**When** wm-spec encounters the error
**Then** it still creates the spec anchor page as a flat doc with all content inline (fallback)
**And** reports which pages failed to create

## Technical Notes

### Skill workflow changes (SKILL.md)

The current Step 3 (Create Spec Document) gains sub-steps:
```
3a. Create spec anchor page (as today, lighter template)
3b. For each locked decision: create Decision page + link `answers`
3c. For each open question: create Decision (draft) page + link `questions`
3d. For each FR: create Task page + link `implements`
3e. For each concept: create Concept page + link `extends`
3f. Update spec anchor `relates_to` with all created page IDs
```

The current Step 5 (Handle Response / approved) gains:
```
5a. Update spec status → approved
5b. For each linked Decision page: update status → approved
5c. For each linked Task page: update status → todo  
5d. For each linked Concept page: update status → reviewed
```

Step 3.5 (Validate) should also validate that all linked pages exist.

### MCP tool calls per spec
For a spec with 3 FRs, 2 decisions, 2 concepts: ~9 page create + ~7 edge link = ~16 MCP calls. Sequential (each depends on previous page ID). Acceptable for human-scale specs.

### ID naming convention
- Decision pages: `wiki:decisions/<spec-name>-<kebab-decision-title>`
- Task pages: `wiki:tasks/<spec-name>-fr-<n>`
- Concept pages: `wiki:concepts/<kebab-concept-name>`

### Edge types to use
| Relationship | Edge type | Direction |
|---|---|---|
| Decision answers spec question | `answers` | Decision → Spec |
| Open question remains | `questions` | Decision → Spec |
| Task implements spec | `implements` | Task → Spec |
| Concept extends concept | `extends` | Concept → Concept |
| Concept used by spec | `references` | Spec → Concept |
| Task depends on task | `depends_on` | Task → Task |

