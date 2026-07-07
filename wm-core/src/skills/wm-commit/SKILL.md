---
name: wm-commit
description: Create conventional commits with wiki validation
---

# Committing Changes

**Announce:** "Using wm-commit."

**Core principle:** VALIDATE WIKI → STAGE → CONVENTIONAL COMMIT.

## Step 1: Validate Wiki

```json
wm_validate.check({})
wm_lint.check({})
```

Fix any issues found.

## Step 2: Stage Changes

```bash
git add -A
git diff --staged --stat
```

## Step 3: Generate Commit Message

Use conventional commit format:

```
feat(<scope>): <description>

- Bullet points of changes
```

## Step 4: Present for Approval

Show staged diff summary and commit message. Wait for user confirmation before committing.
