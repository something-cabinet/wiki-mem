---
name: wm-review
description: Multi-perspective code review with severity-based findings
---

# Code Review

**Announce:** "Using wm-review for [task/scope]."

**Core principle:** MULTI-PERSPECTIVE REVIEW → SEVERITY TRIAGE → FIX P1 → COMMIT.

## Inputs

- Task ID (optional — if provided, reviews against task ACs and spec)
- Current git diff (always)

## Step 1: Gather Review Context

```bash
git diff --stat
git diff
```

If task ID provided:

```json
wm_task.get({"id": "$ARGUMENTS"})
```

If task has spec:

```json
wm_doc.get({"action": "get", "id": "wiki:<spec-path>"})
```

Check for existing patterns and conventions:

```json
wm_search.query({"q": "<feature area>", "type": "doc"})
wm_search.query({"q": "<relevant pattern>", "type": "memory"})
```

## Step 2: Multi-Perspective Review

Review the diff from 5 perspectives. For each, produce findings with severity.

### 2a. Code Quality

- Readability and simplicity
- DRY — duplicated logic
- Error handling — missing or swallowed errors
- Type safety — any `any`, unsafe casts, missing types
- Naming — unclear variable/function names

### 2b. Architecture / Structure

- Separation of concerns — business logic in handlers, UI logic in components
- Coupling — tight dependencies between unrelated modules
- API design — consistent patterns, proper HTTP methods/status codes
- File organization — follows project conventions
- Module boundaries respected, no circular deps

### 2c. Security

- Input validation — user input sanitized
- Auth — proper authorization checks
- Secrets — no hardcoded credentials or tokens
- Data exposure — sensitive data in logs, responses, or error messages

### 2d. Completeness

- Missing tests for new logic
- Edge cases not handled
- Integration gaps — new code not wired into existing flows
- Stubs or TODOs left in code
- ACs from task not fully met (if task provided)

### 2e. Clarity

- Are names descriptive and consistent?
- Are comments meaningful (not redundant)?
- Are functions a reasonable length?
- Does the code follow project conventions?

## Step 3: Triage Findings

Classify each finding:

| Severity | Criteria | Action |
|----------|----------|--------|
| **P1** | Security vuln, data corruption, breaking change, stub shipped | **Blocks commit — must fix** |
| **P2** | Performance issue, architecture concern, missing test | Should fix before commit |
| **P3** | Minor cleanup, naming, style | Record for later |

**Calibration:** Not everything is P1. Severity inflation wastes time. When in doubt, P2.

## Step 4: Report Findings

Group findings by severity:

```
## Review Summary
- **Verdict:** Approve / Changes requested / Blocked
- **P0 findings:** X (Bug or incorrect behavior — blocks acceptance)
- **P1 findings:** X (Blocks commit)
- **P2 findings:** X (Should fix)
- **P3 findings:** X (Nice to have)
```

### Finding Format

```
## [P<severity>] <Title>
- **File:** `path/to/file.ts:42`
- **Issue:** Description of the problem
- **Suggestion:** How to fix or improve
```

## Step 5: Handle Results

### If P1 findings exist — HARD GATE

> ⛔ P1 findings block commit. Fix these first:
> 1. [Finding + suggested fix]
> 2. [Finding + suggested fix]
>
> After fixing, run `/wm-review` again.

Do NOT proceed to commit. Do NOT offer to skip P1.

### If only P2/P3

> ✓ No blocking issues. P2 findings recommended:
> 1. [Finding + suggested fix]
>
> Options:
> - Fix P2s now, then `/wm-commit`
> - Commit as-is: `/wm-commit`
> - Create follow-up task for P2s

### If clean

> ✓ Review passed. No issues found.
>
> Ready: `/wm-commit`

## Step 6: Track Deferred Findings (optional)

If P2 findings are deferred, create a follow-up task:

```json
wm_task.create({"title": "Review follow-up: <summary>", "content": "P2 findings from review of task-<id>:\n- Finding 1\n- Finding 2",
  "priority": "low", "tags": ["review-followup"], "acceptance_criteria": ["Address P2 finding 1", "Address P2 finding 2"]})
```

## Artifact Verification (if task has spec)

For each deliverable in the spec, verify 3 levels:

1. **EXISTS** — file/component/route exists
2. **SUBSTANTIVE** — not a stub (no `return null`, empty handlers, TODO-only implementations)
3. **WIRED** — imported and used in the integration layer

Report:
- ✅ L1+L2+L3: fully wired
- ⚠️ L1+L2 only: created but not integrated → P2
- 🛑 L1 only (stub): exists but empty → P1
- 🛑 Missing: not found → P1

## Checklist

- [ ] Task/spec context read
- [ ] Diff reviewed from 5 perspectives (Code Quality, Architecture, Security, Completeness, Clarity)
- [ ] Findings triaged by severity (P1/P2/P3)
- [ ] P1 findings block commit
- [ ] Artifact verification done (if spec linked)
- [ ] Verdict provided
- [ ] Deferred findings tracked as follow-up tasks (if applicable)

## Red Flags

- Reviewing without understanding the context (task, spec, conventions)
- Ignoring P0/P1 findings — these must be fixed
- Over-indexing on style (P2/P3) while missing correctness issues
- Not checking wiki for existing patterns before suggesting new approaches
- Providing verdict without actionable findings
- Not checking the actual diff (reviewing from memory)
- Severity inflation — calling everything P1
- Skipping security perspective
- Marking stubs as complete

## Final Response Contract

All built-in skills in scope must end with the same user-facing information order: `wm-init`, `wm-spec`, `wm-plan`, `wm-research`, `wm-implement`, `wm-verify`, `wm-doc`, `wm-template`, `wm-extract`, and `wm-commit`.

Required order for the final user-facing response:

1. Goal/result - state what was accomplished.
2. Key details - include the most important supporting context, refs, assumptions, or validation.
3. Next action - recommend a concrete follow-up command only when a natural handoff exists.

Keep this concise for CLI use. Skill-specific content may extend the key-details section, but must not replace or reorder the shared structure.

Out of scope: explaining, syncing, or generating `.claude/skills/*`. Runtime auto-sync already handles platform copies, so this skill source only defines the built-in output contract.

For `wm-review`, the key details should cover:
- severity of findings found (P0/P1/P2/P3)
- whether auto-fixable issues exist
- approval status or blockers

## Related Skills

- `/wm-implement <id>` — Fix findings and continue
- `/wm-commit` — Commit if approved with no P0/P1
- `/wm-verify` — Run final verification

## Next Step Suggestion

```
/wm-implement <task-id>   — Fix findings and continue
/wm-commit                 — Commit if approved with no P0/P1
/wm-verify                 — Run final verification
```
