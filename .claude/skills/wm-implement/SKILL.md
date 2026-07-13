---
name: wm-implement
description: Follow plan, implement changes, check acceptance criteria, and complete the task
---

# Implementing a Task

**Announce:** "Using wm-implement for task [ID]."

**Core principle:** CHECK AC ONLY AFTER WORK IS DONE.

## Inputs

- Task ID
- Existing implementation plan
- Linked spec, docs, templates, and referenced tasks

## Preflight

- Confirm a plan exists; if not, redirect to `/wm-plan <id>` first unless user explicitly overrides
- Read task notes and pending ACs before changing code
- Identify whether the task is standalone or linked to a spec
- If linked to a spec, load the spec only as needed for requirements/AC context
- If the request is to complete an approved spec, route to `/wm-flow @page/<spec-path>` instead
- Decide what verification is required: tests, lint, build, validation, manual checks

## Step 1: Review Task

```json
wm_tasks.get({"taskId": "$ARGUMENTS"})
```

**If task status is "done"** (reopening):

```json
wm_tasks.update({"taskId": "$ARGUMENTS", "status": "in-progress", "appendNotes": "Reopened: <reason>"})
wm_time.start({"taskId": "$ARGUMENTS"})
```

Verify: plan exists, timer running, which ACs pending.

## Step 2: Check Templates

```json
wm_templates.list()
```

If template exists → use it to generate boilerplate.

## Step 3: Set Status

```json
wm_tasks.update({"taskId": "$ARGUMENTS", "status": "in-progress"})
wm_time.start({"taskId": "$ARGUMENTS"})
```

## Step 4: Work Through Plan

For each step:
1. Do the work
2. Check AC (only after done!)
3. Append note

```json
wm_tasks.update({"taskId": "$ARGUMENTS", "checkAc": [1], "appendNotes": "Done: brief description"})
```

### Working Rules

- Append compact progress notes at meaningful checkpoints, not after every tiny edit
- If a step reveals missing context, pause and gather it before continuing
- If task needs page or memory updates, do them as part of completion
- Use `search` to discover relevant sources; use `retrieve` when implementation needs assembled context with citations
- After creating new pages or memory entries, rebuild the index:
  ```json
  # No index-rebuild tool available; skip
  ```

## Step 5: Handle Scope Changes

**Small:** Add AC + note

```json
wm_tasks.update({"taskId": "$ARGUMENTS", "addAc": ["New requirement"], "appendNotes": "Scope: added per user"})
```

**Large:** Stop and ask user.

## Step 6: Validate & Complete

1. Run tests/lint/build

2. Validate task to catch broken refs:

```json
wm_validate({ "entity": "$ARGUMENTS" })
```

3. Capture durable knowledge if the work produced patterns worth remembering
4. Stop timer + mark done:

```json
wm_time.stop({"taskId": "$ARGUMENTS"})
wm_tasks.update({"taskId": "$ARGUMENTS", "status": "done"})
```

**Note:** When task is marked done (or AC is checked), matching ACs in the linked spec document are automatically checked. No manual spec update needed.

## Step 6.5: SDD Workflow (if task has spec)

**Check if task has `spec` field.** If yes, run SDD workflow:

### 1. Get Sibling Tasks

```json
wm_tasks.list({"spec": "<spec-path-from-task>"})
```

Sort siblings by `order`, then by shared title prefix `[<slug>-NN]`.

### 2. Analyze Status

Count tasks by status: `done`, `todo`, `in-progress`.

### 3. Branch Based on Results

**If pending tasks exist:**
```
✓ Task done! This task is part of spec: <spec-path>

Remaining tasks (Y of Z):
- task-YY: [user-auth-02] Title (todo)

Next: /wm-flow @page/<spec-path>
```

**If this is the LAST task (all others done):**
```
✓ Task done! All tasks for <spec-path> complete!

Running SDD verification...
```

Then auto-run:

```json
wm_validate({ "scope": "sdd" })
```

Display SDD Coverage Report:
```
SDD Coverage Report
═══════════════════════════════════════
Spec: <spec-path>
Tasks: X/X complete (100%)
ACs: Y/Z verified

✅ Spec fully implemented!
```

## Step 7: Extract Knowledge (optional)

Before final response, consider whether the work produced guidance future tasks should follow:

- Use a first-class Decision for stable choices (architecture, product behavior, workflow convention)
- Use Memory for concise reusable recall
- Use Pages for long-form explanations

If a quick insight is worth remembering but doesn't warrant a full doc:

```json
wm_memory.add({"title": "<insight>",
  "content": "<2-3 sentence summary>",
  "layer": "project",
  "category": "<pattern|decision|convention>",
  "tags": ["<domain>"]})
```

## Checklist

- [ ] All ACs checked
- [ ] Templates checked and used if applicable
- [ ] Tests pass
- [ ] **Validated (no broken refs)**
- [ ] Durable guidance capture considered
- [ ] Notes added
- [ ] Timer stopped
- [ ] Status = done
- [ ] **SDD workflow handled with sorted sibling summary (if spec linked)**
- [ ] Routed remaining spec work to `/wm-flow` when appropriate
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
- Forgetting to stop timer

## Next Step Suggestion

| Situation | Suggest |
|-----------|---------|
| More tasks in spec | `/wm-flow @page/<spec-path>` |
| All spec tasks done | `/wm-verify` to verify against spec |
| Standalone task done | `/wm-extract` to extract patterns |
| Patterns discovered | `/wm-extract` to document |
