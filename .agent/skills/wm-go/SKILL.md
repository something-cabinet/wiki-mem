---
name: wm-go
description: Execute entire spec pipeline — generate tasks, plan, implement, verify — without review gates
---

**CRITICAL:** Use `task` subagents for delegation. Do NOT create separate sessions or threads unless the user explicitly asks for one.

# Go Mode — Full Pipeline Execution

**Announce:** "Using wm-go for spec [name]."

**Core principle:** SPEC APPROVED → GENERATE TASKS → PLAN → IMPLEMENT ALL → VERIFY → COMMIT.

## Inputs

- Approved spec ref: `@page/specs/<spec-path>`
- Optional: task IDs for a partial wave
- Optional: `--dry-run` to preview tasks without executing

## When to Use

- User has an approved spec and wants to execute everything in one shot
- User says "run all", "go mode", "execute everything", or similar
- The spec is already approved

## When NOT to Use

- Spec is still draft — redirect to `/wm-spec` first
- User wants to review each task individually — use `/wm-plan` + `/wm-implement`
- Spec has unresolved open questions — resolve them first

## Preflight

- Confirm the spec is approved (has `approved` tag)
- Confirm acceptance criteria exist and are testable
- If spec has no tasks yet, they will be generated from requirements

## Phase 1: Validate Spec

```json
wm_doc.get({"path": "specs/<spec-path>"})
```

**Check:**
- Tags include `approved` — if not, STOP: "Spec not approved. Run `/wm-spec` first."
- Has Acceptance Criteria — if empty, STOP: "Spec has no ACs."
- No unresolved open questions marked as blocking

```json
wm_validate.check({"entity": "specs/<spec-path>"})
```

If validation errors → fix or report before continuing.

## Phase 2: Generate Tasks (if needed)

If no tasks exist for this spec:

```json
wm_search.resolve({"q": "<spec-path>"})
```

If no tasks found, generate them using the `--from` pattern but **skip the approval gate**:
- Parse FR-1, FR-2 etc. into logical tasks
- Create tasks with `fulfills` mapping to spec ACs
- Set labels: `from-spec`, `spec:<slug>`

**Report:** "Created X tasks from spec. Starting implementation..."

## Phase 3: Plan + Implement Each Task

Loop through all generated tasks in dependency order (foundational first, dependent last).

For each task:

### 3a. Take ownership + plan

```json
wm_task.update({"id": "<id>", "status": "in-progress"})
wm_time.start({"id": "<id>"})
```

- Research context: follow refs, search related docs/memories, check templates
- Draft and save plan directly (no approval gate)
- Run tests/lint/build after each task

### 3b. Implement

- Work through plan steps
- Check ACs as completed
- Append notes

```json
wm_task.update({"id": "<id>"})
```

### 3c. Complete task

```json
wm_time.stop({"id": "<id>"})
wm_task.update({"id": "<id>", "status": "done"})
```

### 3d. Quick validate

```json
wm_validate.check({"entity": "<id>"})
```

If errors → fix before moving to next task.

**Progress report between tasks:**
> "✓ Task X/Y done: [title]. Continuing..."

## Phase 4: Full Verification

After all tasks complete:

```json
wm_validate.check({"scope": "sdd"})
wm_validate.check({})  # general health check
```

**Report SDD coverage:**

```
SDD Coverage Report
═══════════════════
Spec: specs/<name>
Tasks: X/X complete (100%)
ACs: Y/Z verified
```

If coverage < 100% → identify gaps and fix.

## Phase 5: Commit

Stage all changes and commit with a single conventional commit:

```bash
git add -A
git diff --staged --stat
```

Generate commit message:

```
feat(<scope>): implement <spec-name>

- Task 1: <title>
- Task 2: <title>
- ...
- All ACs verified via SDD
```

**This is the ONE gate in go mode — ask user before committing:**

> Pipeline complete. X tasks done, SDD verified.
>
> Ready to commit:
> ```
> feat(<scope>): implement <spec-name>
> ```
> Proceed? (yes/no/edit)

## Context Budget

If context exceeds ~60% during implementation:

1. Finish the current task
2. Commit completed work so far
3. Report progress and remaining tasks
4. Suggest: "Run `/wm-go @page/specs/<name>` again to continue remaining tasks."

The skill will detect already-done tasks and skip them on re-run.

## Re-run Behavior

When invoked on a spec that already has tasks:

1. List existing tasks linked to the spec
2. Filter to `todo` and `in-progress` only
3. Skip `done` tasks
4. Continue from where it left off

```json
wm_task.list({})
```

## Dry Run Mode

With `--dry-run`:
- Phase 1: validate spec ✓
- Phase 2: generate task preview (don't create) ✓
- Phase 3-5: skip

Show what would be created and ask user to confirm before running for real.

## Error Handling

| Issue | Response |
|-------|----------|
| Build/test failure | Fix and re-run |
| Unfixable error | Mark task blocked, continue with next |
| Spec not approved | Stop and recommend `/wm-spec` |
| Missing tasks | Generate from spec (skip approval gate in go mode) |
| Spec has conflicting requirements | STOP and ask user to clarify |
| Task depends on blocked task | Skip and report at the end |

## Checklist

- [ ] Spec validated (approved, ACs defined)
- [ ] Tasks exist or were generated
- [ ] Each task implemented with ACs checked
- [ ] Full SDD verification passed
- [ ] Build/test/lint passed
- [ ] Changes ready for commit
- [ ] User presented with commit for approval
- [ ] Commit created

## Red Flags

- Running on a draft spec (not approved)
- Skipping validation — go mode is fast but not reckless
- Ignoring build/test failures — fix before continuing
- Not checking re-run state — could duplicate work
- Committing without user confirmation
- Not reporting progress between tasks
- Continuing past context budget limit without checkpointing

## Final Response Contract

All built-in skills in scope must end with the same user-facing information order: `wm-init`, `wm-spec`, `wm-plan`, `wm-research`, `wm-implement`, `wm-verify`, `wm-doc`, `wm-template`, `wm-extract`, and `wm-commit`.

Required order for the final user-facing response:

1. Goal/result - state what was accomplished.
2. Key details - include the most important supporting context, refs, assumptions, or validation.
3. Next action - recommend a concrete follow-up command only when a natural handoff exists.

Keep this concise for CLI use. Skill-specific content may extend the key-details section, but must not replace or reorder the shared structure.

Out of scope: explaining, syncing, or generating `.claude/skills/*`. Runtime auto-sync already handles platform copies, so this skill source only defines the built-in output contract.

For `wm-go`, the key details should cover:
- what was completed, verification results, any remaining work

## Related Skills

- `/wm-commit` — Commit all changes
- `/wm-extract` — Extract patterns from the work
- `/wm-spec` — Start the next spec


## Next Step Suggestion

After completion:

```
/wm-commit   — Commit all changes
/wm-extract  — Extract patterns from the work
/wm-spec     — Start the next spec
```
