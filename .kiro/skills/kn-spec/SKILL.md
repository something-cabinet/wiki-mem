---
name: kn-spec
description: Use when creating a specification document for a feature (SDD workflow)
---

# Creating a Spec Document

Create a specification document for a feature using SDD (Spec-Driven Development).

**Announce:** "Using kn-spec to create spec for [name]."

**Core principle:** EXPLORE DECISIONS → SPEC → REVIEW → APPROVE → THEN PLAN TASKS.

## Inputs

- Feature name
- User requirements, scenarios, constraints, and non-functional expectations
- Related docs/tasks, if any
- Optional: `--skip-explore` to jump straight to spec writing (for trivial features)

## Spec Quality Rules

- Requirements must be testable
- ACs must be observable outcomes, not vague goals
- Scenarios should cover happy path plus at least important edge cases
- Open questions should stay explicit instead of being buried in prose
- If background knowledge is too broad for the spec body, move it into a supporting doc and reference it

---

## Phase 0: Exploring (Socratic Dialog)

Extract decisions from the user BEFORE writing the spec. This prevents the agent from guessing wrong and writing a spec the user didn't want.

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

```json
mcp_knowns_search({ "action": "search", "query": "<feature keywords>", "type": "memory" })
```

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

### 0.5 Transition to Spec

After all gray areas resolved, summarize locked decisions:

> Decisions locked:
> - D1: [summary]
> - D2: [summary]
> - D3: [summary]
>
> Writing spec based on these locked decisions...

---

## Step 1: Get Feature Name

If `$ARGUMENTS` provided, use it as spec name.

If no arguments, ask user:
> What feature are you speccing? (e.g., "user-auth", "payment-flow")

## Step 2: Gather Requirements

Ask user to describe the feature:
> Please describe the feature requirements. What should it do?

Listen for:
- Core functionality
- User stories / scenarios
- Edge cases
- Non-functional requirements

If requirements depend on large domain or architecture context:

- create/update a supporting doc first
- keep the spec focused on product/behavioral requirements
- reference the supporting doc with `@doc/<path>` instead of dumping background material inline

## Step 3: Create Spec Document

```json
mcp_knowns_docs({ "action": "create", "title": "<Feature Name>",
  "description": "Specification for <feature>",
  "folder": "specs",
  "tags": ["spec", "draft"],
  "content": "<spec content>"
})
```

**Spec Template:**

```markdown
## Overview

Brief description of the feature and its purpose.

## Locked Decisions

Decisions extracted during exploring phase:
- D1: [Decision summary]
- D2: [Decision summary]

## Requirements

### Functional Requirements
- FR-1: [Requirement description]
- FR-2: [Requirement description]

### Non-Functional Requirements
- NFR-1: [Performance, security, etc.]

## Acceptance Criteria

- [ ] AC-1: [Testable criterion]
- [ ] AC-2: [Testable criterion]
- [ ] AC-3: [Testable criterion]

## Scenarios

### Scenario 1: [Happy Path]
**Given** [context]
**When** [action]
**Then** [expected result]

### Scenario 2: [Edge Case]
**Given** [context]
**When** [action]
**Then** [expected result]

## Technical Notes

Optional implementation hints or constraints.

## Open Questions

- [ ] Question 1?
- [ ] Question 2?
```

## Step 3.5: Validate Spec

**CRITICAL:** After creating spec, validate to catch issues:

```json
mcp_knowns_validate({ "entity": "specs/<name>" })
```

## Step 4: Ask for Review

Present the spec and ask:
> Please review this spec:
> - **Approve** if requirements are complete
> - **Edit** if you want to modify something
> - **Add more** if requirements are missing

## Step 5: Handle Response

**If approved:**
```json
mcp_knowns_docs({ "action": "update", "path": "specs/<name>",
  "tags": ["spec", "approved"]
})
```

**If edit requested:**
Update the spec based on feedback and return to Step 4.

**If add more:**
Gather additional requirements and update spec.

## Final Response Contract

All built-in skills in scope must end with the same user-facing information order: `kn-init`, `kn-spec`, `kn-plan`, `kn-research`, `kn-implement`, `kn-verify`, `kn-doc`, `kn-template`, `kn-extract`, and `kn-commit`.

Required order for the final user-facing response:

1. Goal/result - state what spec was drafted, revised, approved, or blocked.
2. Key details - include the most important supporting context, refs, open questions, or validation.
3. Next action - recommend a concrete follow-up command only when a natural handoff exists.

Keep this concise for CLI use. Skill-specific content may extend the key-details section, but must not replace or reorder the shared structure.

Out of scope: explaining, syncing, or generating `.claude/skills/*`. Runtime auto-sync already handles platform copies, so this skill source only defines the built-in output contract.

For `kn-spec`, the key details should cover:

- the concrete spec draft or revision
- clear open questions, if any
- approval status
- any validation result or unresolved gaps

## Spillover Rule

If the spec uncovers cross-cutting or general knowledge work:

- create a separate task for that work
- reference it from the spec or generated task set
- keep the spec focused on the feature, not on every general improvement the discussion surfaced

---

## CRITICAL: Next Step Suggestion

**You MUST suggest the next action when a natural follow-up exists. User won't know what to do next.**

After spec is approved:

```
✓ Spec approved: @doc/specs/<name>

Next step — choose one:

1. Task by task (review each step):
   /kn-plan --from @doc/specs/<name>

2. Run all at once (auto pipeline, no review gates):
   /kn-go specs/<name>
```

**Option 1 (`kn-plan --from`):**
- Parse requirements → preview tasks → user approve → create tasks
- Then `/kn-plan <id>` + `/kn-implement <id>` for each task

**Option 2 (`kn-go`):**
- Generate tasks → plan → implement all → verify → commit
- Only stops once at the end for commit confirmation
- Auto-skips done tasks on re-run

---

## Related Skills

- `/kn-plan --from @doc/specs/<name>` - Generate tasks from this spec (manual flow)
- `/kn-go specs/<name>` - Execute entire spec in one run (auto pipeline)
- `/kn-plan <id>` - Plan individual task
- `/kn-verify` - Verify implementation against spec

## Checklist

- [ ] Scope assessed (quick/standard/deep)
- [ ] Gray areas identified and explored (Phase 0)
- [ ] Decisions locked with stable IDs (D1, D2...)
- [ ] Feature name determined
- [ ] Requirements gathered
- [ ] Spec created in specs/ folder
- [ ] Includes: Overview, Locked Decisions, Requirements, ACs, Scenarios
- [ ] User reviewed
- [ ] Status updated (draft → approved)
- [ ] **Next step suggested** (/kn-plan --from or /kn-go)

## Red Flags

- Creating spec without user input
- Skipping Phase 0 for standard/deep scope features
- Batching multiple questions in one message (HARD GATE violation)
- Answering your own questions during exploring
- Skipping review step
- Approving without explicit user confirmation
- **Not suggesting task creation after approval**
- Writing implementation notes instead of requirements
- Leaving ambiguous AC text that cannot be verified later
