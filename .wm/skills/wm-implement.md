---
name: wm-implement
description: Execute a plan, track progress, check acceptance criteria
---

# Implement Task

**Announce:** "Using wm-implement for task [id]."

## Steps

### 1. Take ownership
```
wm_time.start(task_id="wiki:tasks:<task-id>")
```

### 2. Follow the plan
Work through each step of the implementation plan. After each step:

- Update task page with progress
- Check off completed acceptance criteria

### 3. Check ACs
```
wm_page.update(
  id="wiki:tasks:<task-id>",
  checked_ac=[1, 2]
)
```

### 4. Update task status
```
wm_page.update(
  id="wiki:tasks:<task-id>",
  status="done"
)
```

### 5. Record what was learned
If you discovered patterns, decisions, or gotchas, run `wm_extract`.

### 6. Verify
```
wm_validate.check()
wm_lint.check()
wm_time.stop(task_id="wiki:tasks:<task-id>")
```
