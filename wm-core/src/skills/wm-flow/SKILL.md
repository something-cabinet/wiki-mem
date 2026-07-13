---
name: wm-flow
description: Orchestrate an approved spec or task wave through planning, implementation, review, and verification, optionally using parallel sub-agents
---

# Spec Flow Orchestration

**Announce:** "Using wm-flow for spec/task wave [ref]."

**Core principle:** APPROVED SPEC/TASK WAVE → SCHEDULE → PLAN → IMPLEMENT → REVIEW → VERIFY.

## When to Use

- After `/wm-spec` approves a spec and the user wants the work completed end to end
- When the user says "do all tasks", "complete this spec", "orchestrate the work", or similar
- When multiple linked tasks need dependency ordering, ownership boundaries, review, and combined verification

## When NOT to Use

- Draft specs or unresolved product questions → use `/wm-spec`
- A single task with an existing plan → use `/wm-implement <id>`
- Creating tasks only, without execution → use `/wm-plan --from @page/<spec-path>`
- Tiny standalone work → use `/wm-plan --new "<summary>"`

## Inputs

- Spec ref: `@page/<spec-path>` preferred
- Task IDs: one or more explicit tasks for a task wave
- Optional: `--sequential` to force single-threaded execution
- Optional: `--plan-only` to stop after plans and schedule

## Step 1: Startup

1. Read the spec or each explicit task:

```json
wm_doc.get({"path": "<spec-path>"})
```

2. Read the supporting skills:

```json
# Sub-skills are loaded by name — invoke per step below
```

3. Check project state:

```json
wm_project.status()
```

## Step 2: Task Discovery

### For a spec ref

1. List tasks linked to the spec:

```json
wm_search.resolve({"q": "<spec-path>"})
```

2. Sort by `order`, then shared `[slug-NN]` title prefix, then title.

3. If no tasks exist, prompt to create them (do not auto-create without approval unless user explicitly approved task creation in the spec).

### For explicit task IDs

1. Read every task:

```json
wm_task.get({"id": "<id>"})
```

2. Follow refs needed to understand dependencies and verification.

3. Sort by dependency order when visible; otherwise preserve user order.

## Step 3: Parallel Gate

Before spawning workers or implementing in waves, decide what can safely run together.

For each task, note:

| Factor | Assessment |
|--------|------------|
| Dependencies | Does this task depend on another? |
| Write scope | Which files/modules does it touch? |
| Shared risk | Does it touch API/schema/config/generated artifacts? |
| Parallel-safe | Can it run independently? |

**Only run tasks in parallel when:**
- Dependencies are satisfied
- Write scopes are disjoint
- No shared runtime contract is touched

**Default to sequential execution when safety is unclear.**

Report the schedule before implementation:

```markdown
## Execution Schedule
### Wave 1 (parallel-safe)
- task-xxx: [user-auth-01] Add login validation
- task-yyy: [user-auth-02] Add password hashing

### Wave 2 (depends on Wave 1)
- task-zzz: [user-auth-03] Add session management
```

## Step 4: Execution Loop

For each task or parallel-safe wave:

### 4a. Plan

If no saved plan exists or the plan is stale:

```json
wm_task.update({"id": "<id>", "status": "in-progress"})
wm_time.start({"id": "<id>"})
```

Review task context, search related docs, draft the plan.

### 4b. Implement

Execute the plan, check ACs only after work is done:

```json
wm_task.update({"id": "<id>"})
```

### 4c. Review

Review the real diff against the task:

```json
# Use the wm-review skill directly (loaded at startup)
```

### 4d. Fix Findings

- Fix P0 and P1 findings
- Defer P2/P3 with a follow-up note or task if not practical

### 4e. Validate

```json
wm_validate.check({"entity": "<id>"})
```

### 4f. Complete

```json
wm_time.stop({"id": "<id>"})
wm_task.update({"id": "<id>", "status": "done"})
```

### Sub-Agent Orchestration (parallel waves)

When the parallel gate marks tasks as safe and sub-agent tools are available:

**Worker Prompt:**
```
Worker for <TASK_ID> in <SPEC_REF>. Use wm-implement.
Owned scope: <OWNERSHIP_SCOPE>.
Do not revert unrelated changes.
Implement the saved plan, verify it, validate the task, and report changed files, tests, ACs, blockers, and out-of-scope edits.
```

**Reviewer Prompt:**
```
Reviewer for <TASK_ID> in <SPEC_REF>. Use wm-review.
Review the real diff and report verdict, P0/P1/P2/P3 findings with file:line refs, wiring status, fixes, and verification gaps.
```

**After each wave:**
1. Inspect worker output directly
2. Integrate or reject worker changes in the main context
3. Run combined verification for touched areas
4. Re-run review if integration changed reviewed code
5. Close sub-agents after integration

If `--sequential` is set or tools are unavailable, execute the same schedule sequentially in the main context.

## Step 5: Final Verification

Before calling the flow done:

```json
wm_validate.check({"scope": "sdd"})
wm_validate.check({})
```

### Verify:
- All linked spec tasks are done or explicitly blocked
- Task ACs are checked only after implementation
- SDD validation passes for the spec/task set
- Broad verification ran across the integrated diff
- Useful durable memory is captured
- Sub-agents are closed

## Step 6: Summary

```markdown
## Flow Complete
- **Spec/Task Wave:** <ref>
- **Tasks:** X/X complete
- **Review:** X P0, X P1, X P2 findings
- **Verification:** ✅ Passed
- **Next:** /wm-commit
```

## Checklist

- [ ] Spec/tasks read
- [ ] Linked tasks discovered and sorted
- [ ] Parallel gate reported
- [ ] Plans exist for all runnable tasks
- [ ] Implementation completed per task
- [ ] Reviews completed and P0/P1 fixed
- [ ] Combined verification passed
- [ ] SDD validation passed
- [ ] Durable memory captured when useful
- [ ] Sub-agents closed (if used)
- [ ] Next action suggested

## Red Flags

- Running on a draft spec
- Creating tasks without approval
- Parallelizing tasks with shared APIs, schema, config, generated files, migrations, or runtime contracts
- Trusting worker output without inspecting the real diff
- Skipping review before final verification
- Marking the spec done while linked tasks remain unhandled
- Committing or pushing without explicit user request

## Next Step Suggestion

```
/wm-commit   — Commit all completed work
/wm-verify   — Re-run verification for confirmation
/wm-extract  — Extract patterns and learnings
```
