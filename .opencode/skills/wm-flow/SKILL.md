---
name: wm-flow
description: Orchestrate an approved spec or task wave through planning, implementation, review, and verification, optionally using parallel sub-agents
---

**CRITICAL:** Use `task` subagents for delegation. Do NOT create separate sessions or threads unless the user explicitly asks for one.

# Spec Flow Orchestration

**Announce:** "Using wm-flow for spec/task wave [ref]."

**Core principle:** APPROVED SPEC/TASK WAVE → SCHEDULE → PLAN → IMPLEMENT → REVIEW → VERIFY.

**Mode detection:**
- (no flag) → standard mode (gated, review, approval)
- `--fast` → fast mode (skip review, auto-generate tasks, no gates)
- `--fast --resume` → resume incomplete execution, skip done tasks
- `--fast --dry-run` → preview tasks without executing

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
- Optional: `--fast` to enable fast mode (skip review, auto-generate tasks, no gates)
- Optional: `--fast --resume` to resume incomplete execution, skipping done tasks
- Optional: `--fast --dry-run` to preview generated tasks without executing

## Fast Mode

When `--fast` is set, wm-flow switches to an express pipeline:

- **Skip task generation approval**: Tasks are auto-generated from spec FRs without confirmation
- **Skip review gates**: The review step and fix-findings step are omitted
- **Context-budget checkpointing**: When context exceeds ~60%, the current task is completed, work is committed, and the user is prompted to re-run with `--fast --resume`
- **Resume/re-run**: On re-run with `--resume`, done tasks are skipped and execution continues from the first incomplete task
- **Dry-run**: With `--dry-run`, only spec validation and task preview are shown; no work is executed

## Step 1: Startup

1. Read the spec or each explicit task:

```json
wm_doc.get({"action": "get", "id": "wiki:<spec-path>"})
```

2. Read the supporting skills:

```json
# Sub-skills are loaded by name — invoke per step below
```

3. Check project state:

```json
wm_project.status()
```

If `--fast`: validate spec is approved (has `approved` tag) and has acceptance criteria before proceeding. If validation errors, fix or report before continuing.

## Step 2: Task Discovery

### For a spec ref

1. List tasks linked to the spec:

```json
wm_search.resolve({"q": "<spec-path>"})
```

2. Sort by `order`, then shared `[slug-NN]` title prefix, then title.

3. If no tasks exist:
   - Standard mode: prompt to create them (do not auto-create without approval unless user explicitly approved task creation in the spec)
   - `--fast`: auto-generate tasks from spec FRs, parse FR-1, FR-2 etc. into logical tasks, create tasks with `fulfills` mapping to spec ACs, include acceptance_criteria from the spec, set labels `from-spec` and `spec:<slug>`. Report: "Created X tasks from spec. Starting implementation..."

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

When `--fast --resume` is active: check existing tasks linked to the spec, filter to `todo` and `in-progress` only, skip `done` tasks, and continue from the first incomplete task.

### 4a. Plan

If no saved plan exists or the plan is stale:

```json
wm_task.update({"id": "<id>", "status": "in-progress"})
wm_time.start({"id": "<id>"})
```

Review task context, search related docs, draft the plan.

In standard mode: present plan for approval.
In `--fast` mode: draft and save plan directly (no approval gate).

### 4b. Implement

Execute the plan, check ACs only after work is done:

```json
wm_task.update({"id": "<id>"})
```

### 4c. Review

**Standard mode only:**

Review the real diff against the task:

```json
# Use the wm-review skill directly (loaded at startup)
```

**`--fast` mode: skip this step entirely.**

### 4d. Fix Findings

**Standard mode only:**

- Fix P0 and P1 findings
- Defer P2/P3 with a follow-up note or task if not practical

**`--fast` mode: skip this step entirely.**

### 4e. Validate

```json
wm_validate.check({"entity": "<id>"})
```

If errors → fix before moving to next task.

### 4f. Complete

```json
wm_time.stop({"id": "<id>"})
wm_task.update({"id": "<id>", "status": "done"})
```

### 4g. Context-Budget Checkpointing (`--fast` only)

If context exceeds ~60% during implementation:

1. Finish the current task
2. Commit completed work so far
3. Report progress and remaining tasks
4. Suggest: "Run `wm-flow --fast --resume @page/specs/<name>` again to continue remaining tasks"

The skill will detect already-done tasks and skip them on re-run with `--resume`.

### Sub-Agent Orchestration (parallel waves)

When the parallel gate marks tasks as safe, use the `task` tool to spawn subagents that run in their own context and return results. Do NOT create separate sessions or threads for sub-agents.

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

In `--fast` mode: omit reviewer prompt and skip review entirely for sub-agents.

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

## Dry-Run Mode (`--fast --dry-run` only)

With `--dry-run`:
- Validate spec (check approved tag, acceptance criteria)
- Generate task preview (don't create tasks)
- Show what would be created
- Skip all execution phases
- Ask user to confirm before running for real

## Checklist

- [ ] Spec/tasks read
- [ ] Linked tasks discovered and sorted
- [ ] Parallel gate reported
- [ ] Plans exist for all runnable tasks
- [ ] Implementation completed per task
- [ ] Reviews completed and P0/P1 fixed (skipped in `--fast`)
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
- Using `--fast` on a spec that is not approved (must have `approved` tag)
- Using `--fast` without checking re-run state — could duplicate work
- Continuing past context budget limit without checkpointing in `--fast`

## Final Response Contract

All built-in skills in scope must end with the same user-facing information order: `wm-init`, `wm-spec`, `wm-plan`, `wm-research`, `wm-implement`, `wm-verify`, `wm-doc`, `wm-template`, `wm-extract`, and `wm-commit`.

Required order for the final user-facing response:

1. Goal/result - state what was accomplished.
2. Key details - include the most important supporting context, refs, assumptions, or validation.
3. Next action - recommend a concrete follow-up command only when a natural handoff exists.

Keep this concise for CLI use. Skill-specific content may extend the key-details section, but must not replace or reorder the shared structure.

Out of scope: explaining, syncing, or generating `.claude/skills/*`. Runtime auto-sync already handles platform copies, so this skill source only defines the built-in output contract.

For `wm-flow`, the key details should cover:
- tasks completed vs remaining, spec status, blockers

## Related Skills

- `/wm-commit` — Commit all completed work
- `/wm-verify` — Re-run verification for confirmation
- `/wm-extract` — Extract patterns and learnings


## Next Step Suggestion

```
/wm-commit   — Commit all completed work
/wm-verify   — Re-run verification for confirmation
/wm-extract  — Extract patterns and learnings
```

---

## Reference: wm-go (original)

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
wm_doc.get({"action": "get", "id": "wiki:specs/<spec-path>"})
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
