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

## Preflight

- Confirm the correct files are staged/included
- Check whether the commit should reference a task or feature area
- Refuse to commit if changes span unrelated concerns that should be split

## Step 1: Validate Wiki

Before committing, ensure wiki integrity via CLI or MCP tool (`wm:` prefix works across all platforms):

```bash
wm validate       # CLI; MCP: wm_wm_validate_check
wm lint check     # CLI; MCP: wm_wm_lint_check
```

Fix any issues found. Lint fixes can be auto-applied via `wm lint fix`.

## Step 2: Rebuild Search Index

```bash
wm index rebuild  # CLI; MCP: wm_wm_index_rebuild
```

## Step 3: Check Recent Activity

```bash
wm log recent
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

### Rules
- Title lowercase, no period, max 50 characters
- Body explains *why*, not just *what*
- Bullet point each distinct change

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

Format:
```
Ready to commit:

<type>(<scope>): <description>

- Bullet points

Proceed? (yes/no)
```

## Step 7: Commit

```bash
git commit -m "<type>(<scope>): <description>

- Bullet point change"
```

## Final Response Contract

This skill follows the shared output contract. End every response with:

1. **Goal/result** — state whether a commit was proposed, blocked, or created.
2. **Key details** — proposed commit message, relevant diff concerns, approval status.
3. **Next action** — recommend a follow-up command only when a natural handoff exists.

For `wm-commit`, the key details should cover:

- the proposed commit title
- 1 short body explaining why
- any concerns about the staged diff
- a clear approval prompt

## Checklist

- [ ] Wiki validated
- [ ] Lint checked/fixed
- [ ] Search index rebuilt
- [ ] Recent logs reviewed
- [ ] Changes staged
- [ ] Conventional commit message generated
- [ ] User approved

## Abort Conditions

- Nothing staged
- Staged diff includes unrelated work that should be split
- Wiki validation has errors that can't be auto-fixed
- User has not explicitly approved the final message

## Red Flags

- Committing with wiki validation errors
- Committing without rebuilding index
- Pushing without user confirmation
- Using vague commit messages without scope
- Including unintended files (node_modules, secrets, build artifacts)
- "Co-Authored-By" or "Generated with AI" lines
- Title over 50 characters or with period

## Next Step Suggestion

- After a successful commit tied to active work: `/wm-verify`
- After a successful standalone commit: `/wm-extract`
- No command while waiting for approval — wait for user
