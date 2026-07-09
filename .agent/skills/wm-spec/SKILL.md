---
name: wm-spec
description: Create specification documents using Spec-Driven Development (SDD)
---

# Creating a Spec Document

**Announce:** "Using wm-spec to create spec for [name]."

**Core principle:** EXPLORE DECISIONS → SPEC → REVIEW → APPROVE.

## Inputs

- Feature name or problem to specify
- Optional: known constraints, existing research, related specs

## Phase 0: Socratic Exploration

Before writing, explore gray areas to avoid prematurely committing to incomplete requirements.

- Assess scope: quick / standard / deep
- Identify gray areas — ambiguous requirements, unstated tradeoffs, conflicting constraints
- Ask **one question at a time** — allow user to respond before proceeding
- Lock each decision with a `D-` ID once resolved

### Exploration Topics

| Topic | Questions to explore |
|-------|---------------------|
| **Scope** | What is in/out? What are the boundaries? |
| **Users** | Who uses this? What are the personas? |
| **Behavior** | What should happen in normal/error/edge cases? |
| **Data** | What data is involved? Where does it live? |
| **Dependencies** | What does this depend on? What depends on this? |
| **Tradeoffs** | What tradeoffs exist? Performance vs clarity? Speed vs safety? |

## Step 1: Create Spec Page

```json
wm_wm_page_create({
  "id": "specs/<feature-name>",
  "title": "<Feature Name>",
  "tags": ["<search-keyword-1>", "<search-keyword-2>"],  # Use specific search keywords (e.g., "auth", "mcp", "graph"), not metadata
  "content": "<spec content>"
})
```

### Spec Template

```markdown
## Overview
[One-paragraph description of the feature]

## Locked Decisions
- **D-1:** [Decision title] — [Rationale]
- **D-2:** [Decision title] — [Rationale]

## Requirements
### Functional
- FR-1: [Functional requirement]
- FR-2: [Functional requirement]

### Non-Functional
- NFR-1: [Non-functional requirement]

## Acceptance Criteria
- AC-1: [Acceptance criterion]
- AC-2: [Acceptance criterion]

## Scenarios
### Happy Path
[Steps and expected outcome]

### Edge Cases
[List of edge cases and expected behavior]

## Open Questions
- [Question 1]
- [Question 2]
```

## Step 2: Validate Spec

```json
wm_wm_validate_check({ "entity": "specs/<feature-name>" })
```

Fix any broken refs or structural issues found.

## Step 3: Review & Approve

Present the spec for user review. Key questions:
- Are all functional requirements captured?
- Are edge cases covered?
- Are the acceptance criteria testable?
- Are locked decisions truly resolved?

On approval, set status:

```json
wm_wm_page_update({ "id": "specs/<feature-name>", "status": "approved" })
```

## Checklist

- [ ] Gray areas explored via Socratic questions (one at a time)
- [ ] Decisions locked with D- IDs
- [ ] Spec covers: Overview, Requirements, ACs, Scenarios
- [ ] Open questions documented
- [ ] Spec validated
- [ ] User reviewed and approved
- [ ] Tags updated to `approved`

## Red Flags

- Rushing to write before exploring gray areas
- Asking multiple questions at once — always ask one at a time
- Spec without testable acceptance criteria
- Spec without edge case scenarios
- Skipping validation before review

## Next Step Suggestion

After approval:

```
/wm-flow @page/specs/<feature-name>   — Orchestrate full implementation
/wm-plan --from @page/specs/<feature-name> — Generate tasks from spec
```
