---
name: wm-review
description: Multi-perspective code review with severity-based findings
---

# Code Review

**Announce:** "Using wm-review for [task/scope]."

**Core principle:** STRUCTURE → CORRECTNESS → CLARITY → CONSISTENCY.

## Inputs

- Task ID, spec ref, or diff scope to review
- Optional: specific focus areas or concerns

## Review Perspectives

Review changed code through these lenses:

| Perspective | What to check |
|-------------|---------------|
| **Structure** | Architecture fit, module boundaries, dependency direction |
| **Correctness** | Edge cases, error handling, race conditions, type safety |
| **Clarity** | Naming, comments, function length, readability |
| **Consistency** | Project conventions, pattern reuse, style |

## Step 1: Gather Context

```json
wm:task.get({ "taskId": "$ARGUMENTS" })
wm:page.get({ "id": "CONVENTIONS", "smart": true })
```

Check for existing patterns and conventions:

```json
wm:search.query({ "query": "<relevant pattern>", "type": "page" })
wm:search.query({ "query": "<relevant pattern>", "type": "memory" })
```

## Step 2: Review Each Perspective

### Structure
- Does the code fit the existing architecture?
- Are module boundaries respected?
- Is dependency direction correct (no circular deps)?

### Correctness
- Are all edge cases handled?
- Is error handling appropriate?
- Are there race conditions or type safety issues?

### Clarity
- Are names descriptive and consistent?
- Are comments meaningful (not redundant)?
- Are functions a reasonable length?

### Consistency
- Does the code follow project conventions?
- Does it reuse existing patterns?
- Is the style consistent with surrounding code?

## Step 3: Report Findings

Group findings by severity:

| Severity | Label | Action |
|----------|-------|--------|
| **P0** | Bug or incorrect behavior | **Must fix** — blocks acceptance |
| **P1** | Design or clarity issue | **Should fix** — address before merge |
| **P2** | Style or minor concern | **Nice to fix** — defer if busy |
| **P3** | Nitpick or suggestion | **Consider** — optional improvement |

### Finding Format

```
## [P<severity>] <Title>
- **File:** `path/to/file.ts:42`
- **Issue:** Description of the problem
- **Suggestion:** How to fix or improve
```

## Step 4: Summary & Verdict

Provide a summary verdict:

```markdown
## Review Summary
- **Verdict:** Approve / Changes requested / Blocked
- **P0 findings:** X
- **P1 findings:** Y
- **P2 findings:** Z
- **P3 findings:** W
- **Overall:** [Brief assessment]
```

## Checklist

- [ ] Task/spec context read
- [ ] Conventions checked
- [ ] Structure reviewed
- [ ] Correctness reviewed
- [ ] Clarity reviewed
- [ ] Consistency reviewed
- [ ] Findings grouped by severity (P0/P1/P2/P3)
- [ ] Verdict provided

## Red Flags

- Reviewing without understanding the context (task, spec, conventions)
- Ignoring P0 findings — these must be fixed
- Over-indexing on style (P2/P3) while missing correctness issues
- Not checking wiki for existing patterns before suggesting new approaches
- Providing verdict without actionable findings

## Next Step Suggestion

```
/wm-implement <task-id>   — Fix findings and continue
/wm-commit                 — Commit if approved with no P0/P1
/wm-verify                 — Run final verification
```
