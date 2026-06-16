---
name: wm-commit
description: Verify work, record knowledge, and commit with conventional format
---

# Commit

**Announce:** "Using wm-commit."

## Steps

### 1. Verify wiki health
```
wm_lint.check()
wm_validate.check()
```

### 2. Run extract
```
wm-extract <task-id>
```

### 3. Stage and commit
```bash
git add -A
git diff --staged --stat
```

### 4. Generate commit message
```
feat(<scope>): <description>

- <detail>
- <detail>
```

### 5. Ask user for approval
> Ready to commit with message above? (yes/no/edit)
