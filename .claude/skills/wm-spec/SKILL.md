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
- Optional: `--skip-explore` to jump straight to spec writing (for trivial features)

## Spec Quality Rules

- Requirements must be testable
- ACs must be observable outcomes, not vague goals
- Scenarios should cover happy path plus at least important edge cases
- Open questions should stay explicit instead of being buried in prose
- If background knowledge is too broad for the spec body, move it into a supporting doc and reference it

## Phase 0: Socratic Exploration

Before writing, explore gray areas to avoid prematurely committing to incomplete requirements.

### 0.1 Scope Assessment

Assess from the request + a quick project scan:

- **Quick** — bounded, low ambiguity (rename a flag, tweak a label). Skip to Step 1 (or use `--skip-explore`).
- **Standard** — normal feature with decisions to extract. Run full Phase 0.
- **Deep** — cross-cutting, strategic, or highly ambiguous. Run Phase 0 with extra depth.

### 0.2 Domain Classification

Classify what is being built — this determines which gray areas to probe:

| Type | What it is | Example |
|------|-----------|---------|
| **SEE** | Something users look at | UI, dashboard, layout |
| **CALL** | Something callers invoke | API, CLI command, webhook |
| **RUN** | Something that executes | Background job, script, service |
| **READ** | Something users read | Docs, emails, reports |
| **ORGANIZE** | Something being structured | Data model, file layout, taxonomy |

One feature can span types (e.g., SEE + CALL).

### 0.3 Gray Area Identification

Generate 2–4 gray areas for this feature. A gray area is a decision that:
- Affects implementation specifics
- Was not stated in the request
- Would force the planner to make an assumption without it

**Quick codebase scout** (grep only — no deep analysis):
- Check what already exists that's related
- Search for past decisions and patterns on this topic
- Annotate options with what the codebase already has

**Filter OUT:**
- Technical implementation details (architecture, library choices) — that's planning's job
- Performance concerns
- Scope expansion (new capabilities not requested)

### 0.4 Socratic Exploration

<HARD-GATE>
Ask ONE question at a time. Wait for the user's response before asking the next.
Do NOT batch questions. Do NOT answer your own questions.
Do NOT proceed to spec writing until all gray areas have been discussed.
</HARD-GATE>

**Rules:**
1. One question per message — never bundled
2. Single-select multiple choice preferred over open-ended
3. Start broad (what/why/for whom) then narrow (constraints, edge cases)
4. 3–4 questions per gray area, then checkpoint:
   > "More questions about [area], or move on? (Remaining: [unvisited areas])"

**Scope creep response** — when user suggests something outside scope:
> "[Feature X] is a new capability — will be a separate work item. Noted. Back to [current area]: [question]"

**Decision locking** — after each gray area is resolved:
> "Lock decision D[N]: [summary]. Confirmed?"

Assign stable IDs: D1, D2, D3... These IDs will be referenced in the spec.

### Exploration Topics

| Topic | Questions to explore |
|-------|---------------------|
| **Scope** | What is in/out? What are the boundaries? |
| **Users** | Who uses this? What are the personas? |
| **Behavior** | What should happen in normal/error/edge cases? |
| **Data** | What data is involved? Where does it live? |
| **Dependencies** | What does this depend on? What depends on this? |
| **Tradeoffs** | What tradeoffs exist? Performance vs clarity? Speed vs safety? |

### 0.5 Transition to Spec

After all gray areas resolved, summarize locked decisions:

> Decisions locked:
> - D1: [summary]
> - D2: [summary]
> - D3: [summary]
>
> Writing spec based on these locked decisions...

## Step 1: Create Spec Page

```json
wm_docs.create({"title": "<Feature Name>",
  "folder": "specs",
  "tags": ["<search-keyword-1>", "<search-keyword-2>"],
  "content": "<spec content>"})
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
- [ ] AC-1: [Observable, testable criterion]
- [ ] AC-2: [Observable, testable criterion]

## Scenarios
### Happy Path
**Given** [context]
**When** [action]
**Then** [expected result]

### Edge Cases
**Given** [context]
**When** [action]
**Then** [expected result]

## Open Questions
- [ ] Question 1?
- [ ] Question 2?
```

## Step 2: Validate Spec

```json
wm_validate({ "entity": "specs/<feature-name>" })
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
wm_docs.update({"path": "specs/<feature-name>", "tags": ["spec", "approved"]})
```

## Spillover Rule

If the spec uncovers cross-cutting or general knowledge work:
- Create a separate task for that work
- Reference it from the spec or generated task set
- Keep the spec focused on the feature, not on every general improvement the discussion surfaced

## Checklist

- [ ] Scope assessed (quick/standard/deep)
- [ ] Domain classified (SEE/CALL/RUN/READ/ORGANIZE)
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
- Allowing scope creep into the spec without spillover to separate tasks

## Next Step Suggestion

After approval:

```
/wm-flow @page/specs/<feature-name>   — Orchestrate full implementation
/wm-plan --from @page/specs/<feature-name> — Generate tasks from spec
```
