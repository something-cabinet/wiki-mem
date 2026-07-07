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

## Step 2: Rebuild Index

```json
wm_index.rebuild({})
```

## Step 3: Stage Changes

```bash
git add -A
git diff --staged --stat
```

## Step 4: Generate Commit Message

Use conventional commit format:

```
feat(<scope>): <description>

- Bullet points of changes
```

## Step 5: Present for Approval

Show staged diff summary and commit message. Wait for user confirmation before committing.
