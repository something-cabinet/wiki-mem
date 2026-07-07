---
name: wm-implement
description: Follow plan, implement changes, check acceptance criteria
---

# Implementing a Task

**Announce:** "Using wm-implement for task [ID]."

**Core principle:** CHECK AC ONLY AFTER WORK IS DONE.

## Step 1: Review Task

```json
wm_task.get({ "taskId": "<id>" })
```

## Step 2: Set Status

```json
wm_task.update({ "taskId": "<id>", "status": "in-progress" })
wm_time.start({ "taskId": "<id>" })
```

## Step 3: Work Through Plan

For each step: implement, then check AC.

```json
wm_task.update({ "taskId": "<id>", "checkAc": [1], "appendNotes": "Done: ..." })
```

After writing new pages or memory entries, rebuild the search index:

```json
wm_index.rebuild({})
```

## Step 4: Validate

```json
wm_validate.check({ "entity": "<id>" })
```

## Step 5: Complete

```json
wm_time.stop({ "taskId": "<id>" })
wm_task.update({ "taskId": "<id>", "status": "done" })
```

If task has spec ref, run SDD completion workflow (check sibling tasks, run full verification if last task).
