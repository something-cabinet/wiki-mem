---
name: wm-commit
description: Create conventional commits with wiki validation and verification
---

# Committing Changes

**Announce:** "Using wm-commit."

**Core principle:** VALIDATE WIKI → STAGE → CONVENTIONAL COMMIT.

## Inputs

- Current working tree with changes to commit
- Optional: suggested commit scope and description

## Step 1: Validate Wiki

Before committing, ensure wiki integrity:

```json
validate.check({})
lint.check({})
```

Fix any issues found. Lint fixes can be auto-applied:

```json
lint.fix({})
```

## Step 2: Rebuild Search Index

```json
index.rebuild({})
```

## Step 3: Check Recent Activity

```json
log.recent({ "limit": 10 })
```

Review recent changes to ensure context is fresh and nothing was missed.

## Step 4: Stage Changes

```bash
git add -A
git diff --staged --stat
```

Review the staged diff summary. Verify no unintended files are included.

## Step 5: Generate Commit Message

Use conventional commit format:

```
<type>(<scope>): <description>

- Bullet points of changes
```

### Types
| Type | Usage |
|------|-------|
| `feat` | New feature |
| `fix` | Bug fix |
| `docs` | Documentation changes |
| `refactor` | Code restructuring |
| `test` | Test additions/changes |
| `chore` | Maintenance, tooling, config |
| `perf` | Performance improvement |

### Examples
```
feat(auth): add refresh token rotation

- Add refresh token expiry check
- Auto-rotate on token refresh
- Add migration for existing tokens
```

```
docs(specs): add user-auth spec

- Functional requirements FR-1–FR-5
- Edge case scenarios for token expiry
- Locked decisions D-1 and D-2
```

## Step 6: Present for Approval

Show staged diff summary and commit message. **Wait for user confirmation before committing.**

## Checklist

- [ ] Wiki validated
- [ ] Lint checked/fixed
- [ ] Search index rebuilt
- [ ] Recent logs reviewed
- [ ] Changes staged
- [ ] Conventional commit message generated
- [ ] User approved

## Red Flags

- Committing with wiki validation errors
- Committing without rebuilding index
- Pushing without user confirmation
- Using vague commit messages without scope
- Including unintended files (node_modules, secrets, build artifacts)

## Next Step Suggestion

After commit:

```
/wm-extract   — Extract patterns or decisions from the work
/wm-spec      — Start next spec
/wm-go        — Continue with next pipeline
```

