---
name: kn-implement
description: Use when implementing a task - follow the plan, check ACs, track progress
---

# Implementing a Task

Execute the implementation plan, track progress, and complete the task.

**Announce:** "Using kn-implement for task [ID]."

**Core principle:** CHECK AC ONLY AFTER WORK IS DONE.

## Inputs

- Task ID
- Existing implementation plan
- Linked spec, docs, templates, and referenced tasks

## Preflight

- Confirm a plan exists; if not, redirect to `/kn-plan <id>` first unless user explicitly overrides
- Read task notes and pending ACs before changing code
- Identify whether the task is standalone or linked to a spec
- Decide what verification is required: tests, lint, build, validation, manual checks

## Step 1: Review Task

```json
mcp_knowns_tasks({ "action": "get", "taskId": "$ARGUMENTS" })
```

**If task status is "done"** (reopening):
```json
mcp_knowns_tasks({ "action": "update", "taskId": "$ARGUMENTS",
  "status": "in-progress",
  "appendNotes": "Reopened: <reason>"
})
mcp_knowns_time({ "action": "start", "taskId": "$ARGUMENTS" })
```

Verify: plan exists, timer running, which ACs pending.

## Step 2: Check Templates

```json
mcp_knowns_templates({ "action": "list" })
```

If template exists → use it to generate boilerplate.

## Step 3: Work Through Plan

For each step:
1. Do the work
2. Check AC (only after done!)
3. Append note

```json
mcp_knowns_tasks({ "action": "update", "taskId": "$ARGUMENTS",
  "checkAc": [1],
  "appendNotes": "Done: brief description"
})
```

Working rules:

- Append compact progress notes at meaningful checkpoints, not after every tiny edit
- If a step reveals missing context, pause implementation and gather it before continuing
- If the task needs docs or template changes, do them as part of completion, not as an afterthought
- Use `search` to discover relevant sources; use `retrieve` when implementation needs assembled context with citations for docs, tasks, and memories.
- Prefer MCP `mcp_knowns_search({ "action": "retrieve", "query": "<keywords>" })` for retrieval; fall back to CLI `knowns retrieve "<keywords>" --json` if MCP is unavailable.

## Step 4: Handle Scope Changes

**Small:** Add AC + note
```json
mcp_knowns_tasks({ "action": "update", "taskId": "$ARGUMENTS",
  "addAc": ["New requirement"],
  "appendNotes": "Scope: added per user"
})
```

**Large:** Stop and ask user.

## Step 5: Validate & Complete

1. Run tests/lint/build
2. **Validate task** to catch broken refs (uses entity filter to save tokens):

```json
mcp_knowns_validate({ "entity": "$ARGUMENTS" })
```

3. Add implementation notes (use `appendNotes`, NOT `notes`!)
4. Stop timer + mark done

```json
mcp_knowns_time({ "action": "stop", "taskId": "$ARGUMENTS" })
mcp_knowns_tasks({ "action": "update", "taskId": "$ARGUMENTS",
  "status": "done"
})
```

**Note:** When task is marked done (or AC is checked), matching ACs in the linked spec document are automatically checked. No manual spec update needed.

## Step 5.5: SDD Workflow (if task has spec)

**Check if task has `spec` field.** If yes, run SDD workflow:

### 1. Get Sibling Tasks

```json
mcp_knowns_tasks({ "action": "list", "spec": "<spec-path-from-task>" })
```

### 2. Analyze Status

Count tasks by status:
- `done`: completed tasks
- `todo` / `in-progress`: pending tasks

### 3. Branch Based on Results

**If pending tasks exist:**
```
✓ Task done! This task is part of spec: specs/xxx

Remaining tasks (Y of Z):
- task-YY: Title (todo)
- task-ZZ: Title (in-progress)

Next: /kn-plan <first-todo-id>
```

**If this is the LAST task (all others done):**
```
✓ Task done! All tasks for specs/xxx complete!

Running SDD verification...
```

Then auto-run:
```json
mcp_knowns_validate({ "scope": "sdd" })
```

Display SDD Coverage Report:
```
SDD Coverage Report
═══════════════════════════════════════
Spec: specs/xxx
Tasks: X/X complete (100%)
ACs: Y/Z verified

✅ Spec fully implemented!
```

## Step 6: Extract Knowledge (optional)

If patterns discovered: `/kn-extract`

If a quick insight is worth remembering but doesn't warrant a full doc:
```json
mcp_knowns_memory({ "action": "add", "title": "<insight>",
  "content": "<2-3 sentence summary>",
  "layer": "project",
  "category": "<pattern|decision|convention>",
  "tags": ["<domain>"]
})
```

## Final Response Contract

All built-in skills in scope must end with the same user-facing information order: `kn-init`, `kn-spec`, `kn-plan`, `kn-research`, `kn-implement`, `kn-verify`, `kn-doc`, `kn-template`, `kn-extract`, and `kn-commit`.

Required order for the final user-facing response:

1. Goal/result - state what was implemented, confirmed, or what remains blocked.
2. Key details - include the most important supporting context, verification, refs, or spec status.
3. Next action - recommend a concrete follow-up command only when a natural handoff exists.

Keep this concise for CLI use. Skill-specific content may extend the key-details section, but must not replace or reorder the shared structure.

Out of scope: explaining, syncing, or generating `.claude/skills/*`. Runtime auto-sync already handles platform copies, so this skill source only defines the built-in output contract.

For `kn-implement`, the key details should cover:

- whether the task is done or what remains
- tests, validation, lint, or build status
- any spec-related follow-up or remaining sibling-task context

---

## CRITICAL: Next Step Suggestion

**You MUST suggest the next action when a natural follow-up exists. User won't know what to do next.**

After task completion, check for:

1. **More tasks from same spec?**
   ```json
   mcp_knowns_tasks({ "action": "list", "spec": "<spec-path>", "status": "todo" })
   ```

2. **Suggest based on context:**

| Situation | Suggest |
|-----------|---------|
| More tasks in spec | "Next: `/kn-plan <next-task-id>` for [task title]" |
| All spec tasks done | "All tasks complete! Run `/kn-verify` to verify against spec" |
| Standalone task | "Task done. Run `/kn-extract` to extract patterns, or `/kn-commit` to commit" |
| Patterns discovered | "Consider `/kn-extract` to document this pattern" |

**Example output:**
```
✓ Task #43 complete!

Next task from @doc/specs/user-auth:
→ Task #44: Add refresh token rotation

Run: /kn-plan 44
```

---

## Related Skills

- `/kn-plan <id>` - Create plan before implementing
- `/kn-verify` - Verify all tasks against spec
- `/kn-extract` - Extract patterns to docs
- `/kn-commit` - Commit with verification

## Checklist

- [ ] All ACs checked
- [ ] Tests pass
- [ ] **Validated (no broken refs)**
- [ ] Notes added
- [ ] Timer stopped
- [ ] Status = done
- [ ] **SDD workflow handled (if spec linked)**
- [ ] **Next step suggested**

## Red Flags

- Checking AC before work done
- Skipping tests
- Skipping validation
- Using `notes` instead of `appendNotes`
- Marking done without verification
- **Not checking sibling tasks when spec linked**
- **Not running SDD verify when spec complete**
- **Not suggesting next step**
- Implementing from a vague task without clarifying plan/context
- Silently expanding scope instead of asking
