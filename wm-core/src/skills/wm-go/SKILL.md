---
name: wm-go
description: Execute entire spec pipeline — generate tasks, plan, implement, verify — without review gates
---

# Go Mode

**Announce:** "Using wm-go for spec [name]."

**Core principle:** SPEC APPROVED → GENERATE TASKS → IMPLEMENT ALL → VERIFY → COMMIT.

## Inputs

- Approved spec ref: `@page/specs/<spec-path>`
- Optional: task IDs for a partial wave

## Preflight

- Confirm the spec is approved (has `approved` tag)
- Confirm acceptance criteria exist and are testable
- If spec has no tasks yet, they will be generated from requirements

## Step 1: Validate Spec

```json
page.get({ "id": "specs/<spec-path>", "smart": true })
```

Check:
- `status` is `approved`
- Acceptance criteria (AC-1, AC-2, etc.) are defined
- Requirements are scoped to implementable units

## Step 2: Generate Tasks (if needed)

If no tasks exist for this spec:

```json
search.resolve({ "ref": "@page/specs/<spec-path>{implements}", "direction": "inbound", "entityTypes": "task" })
```

If no tasks found, generate them using the `--from` pattern:
- Parse FR-1, FR-2 etc. into logical tasks
- Create tasks with `fulfills` mapping to spec ACs
- Set labels: `from-spec`, `spec:<slug>`

## Step 3: Plan + Implement Each Task

Loop through each task in order:

For each task:
1. Plan directly (skip approval gate)
2. Start timer
3. Implement
4. Check ACs
5. Capture notes
6. Stop timer
7. Mark done

```json
task.update({ "taskId": "<id>", "status": "in-progress" })
time.start({ "taskId": "<id>" })
// ... implement ...
task.update({ "taskId": "<id>", "checkAc": [1], "appendNotes": "Done: ..." })
time.stop({ "taskId": "<id>" })
task.update({ "taskId": "<id>", "status": "done" })
```

## Step 4: Full Verification

After all tasks complete:

```json
validate.check({ "scope": "sdd" })
lint.check({})
```

Review SDD coverage report and fix any issues.

## Step 5: Rebuild Index

```json
index.rebuild({})
```

## Step 6: Commit

Stage all changes, generate conventional commit message, present for user approval.

## Re-run Behavior

- Already-done tasks are skipped
- Continues from where it left off
- If a task is in `in-progress` status, re-plan and continue

## Error Handling

| Issue | Response |
|-------|----------|
| Build/test failure | Fix and re-run |
| Unfixable error | Mark task blocked, continue with next |
| Spec not approved | Stop and recommend `/wm-spec` |
| Missing tasks | Generate from spec, ask confirmation |

## Checklist

- [ ] Spec validated (approved, ACs defined)
- [ ] Tasks exist or were generated
- [ ] Each task implemented with ACs checked
- [ ] Full SDD verification passed
- [ ] Index rebuilt
- [ ] Changes ready for commit
- [ ] User presented with commit for approval

## Red Flags

- Running on a draft spec (not approved)
- Skipping validation — go mode is fast but not reckless
- Ignoring build/test failures — fix before continuing
- Not checking re-run state — could duplicate work
- Committing without user confirmation

## Next Step Suggestion

After completion:

```
/wm-commit   — Commit all changes
/wm-extract  — Extract patterns from the work
/wm-spec     — Start the next spec
```

