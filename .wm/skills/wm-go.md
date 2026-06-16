---
name: wm-go
description: Full pipeline — generate tasks, plan, implement, verify, commit. No review gates.
---

# Go Mode

**Announce:** "Using wm-go for spec [name]."

**Core principle:** SPEC → TASKS → PLAN → IMPLEMENT → VERIFY → COMMIT.

## Process

### 1. Validate spec
- Read the spec page — check it has acceptance criteria
- Validate graph health

### 2. Generate tasks
Create task pages from spec requirements. Each task gets its own page with `type: task` and `implements` edges to the spec.

### 3. Plan + implement each task
For each task in dependency order:
- `wm-plan <task-id>` — gather context, create plan
- `wm-implement <task-id>` — execute, track ACs
- `wm-extract <task-id>` — save learnings

### 4. Full verification
```
wm_validate.check()
wm_lint.check()
```

### 5. Commit
```
wm-commit
```
