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
- If linked to a spec, load the spec only as needed for requirements/AC context
- If the request is to complete an approved spec, route to `/wm-flow @page/<spec-path>` instead
- Decide what verification is required: tests, lint, build, validation, manual checks

## Step 1: Review Task

```json
wm_task.get({ "taskId": "$ARGUMENTS" })
```

**If task status is "done"** (reopening):

```json
wm_task.update({ "taskId": "$ARGUMENTS", "status": "in-progress", "appendNotes": "Reopened: <reason>" })
wm_time.start({ "taskId": "$ARGUMENTS" })
```

## Step 2: Set Status

```json
wm_task.update({ "taskId": "$ARGUMENTS", "status": "in-progress" })
wm_time.start({ "taskId": "$ARGUMENTS" })
```

## Step 3: Work Through Plan

For each step:
1. Do the work
2. Check AC (only after done!)
3. Append note

```json
wm_task.update({ "taskId": "$ARGUMENTS", "checkAc": [1], "appendNotes": "Done: brief description" })
```

### Working Rules

- Append compact progress notes at meaningful checkpoints
- If a step reveals missing context, pause and gather it before continuing
- If task needs page or memory updates, do them as part of completion
- After creating new pages or memory entries, rebuild the index:

```json
wm_index.rebuild({})
```

## Step 4: Handle Scope Changes

**Small:** Add AC + note

```json
wm_task.update({ "taskId": "$ARGUMENTS", "addAc": ["New requirement"], "appendNotes": "Scope: added per user" })
```

**Large:** Stop and ask user.

## Step 5: Validate & Complete

1. Run tests/lint/build

2. Validate task to catch broken refs:

```json
wm_validate.check({ "entity": "$ARGUMENTS" })
```

3. Capture durable knowledge if the work produced patterns worth remembering
4. Stop timer + mark done:

```json
wm_time.stop({ "taskId": "$ARGUMENTS" })
wm_task.update({ "taskId": "$ARGUMENTS", "status": "done" })
```

## Step 5.5: SDD Workflow (if task has spec)

**Check if task has `spec` field.** If yes, run SDD workflow:

### 1. Get Sibling Tasks

```json
wm_task.list({ "spec": "<spec-path-from-task>" })
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
wm_validate.check({ "scope": "sdd" })
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

## Step 6: Capture Durable Knowledge (optional)

Before final response, consider whether the work produced guidance future tasks should follow:

- Use a first-class Decision for stable choices (architecture, product behavior, workflow convention)
- Use Memory for concise reusable recall
- Use Pages for long-form explanations

## Checklist

- [ ] All ACs checked
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

## Next Step Suggestion

| Situation | Suggest |
|-----------|---------|
| More tasks in spec | `/wm-flow @page/<spec-path>` |
| All spec tasks done | `/wm-verify` to verify against spec |
| Standalone task done | `/wm-extract` to extract patterns |
| Patterns discovered | `/wm-extract` to document |
