---
name: wm-commit
description: Create conventional commits with wiki validation and verification
---

# Committing Changes

**Announce:** "Using wm-commit."

**Core principle:** VERIFY BEFORE COMMITTING — check staged changes, ask for confirmation.

## Inputs

- Current working tree with changes to commit
- Optional: suggested commit scope and description

## Preflight

- Confirm the correct files are staged
- Check whether the commit should reference a task or feature area
- Refuse to commit if the staged diff looks unrelated or mixed across multiple concerns

## Step 1: Validate Wiki

Before committing, ensure wiki integrity:

```json
wm_validate.check({})
```

Fix any issues found.

## Step 2: Check Project State

```json
wm_project.status()
```

## Step 3: Review Staged Changes

```bash
git status
git diff --staged
git diff --staged --stat
```

Review the staged diff. Verify no unintended files are included.

## Step 4: Generate Commit Message

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

### Format Rules
- Title lowercase, no period, max 50 chars
- Body explains *why*, not just *what*

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

## Step 5: Present for Approval

Show staged diff summary and commit message:

```
Ready to commit:

feat(auth): add JWT token refresh

- Added refresh token endpoint

Proceed? (yes/no/edit)
```

**Wait for user confirmation before committing.**

## Commit Guidelines

- Only commit staged files
- NO "Co-Authored-By" lines
- NO "Generated with Claude Code" ads
- Ask before committing

## Abort Conditions

- Nothing staged
- Staged diff includes unrelated work that should be split
- User has not explicitly approved the final message
- Wiki validation errors are present (fix first)

## Checklist

- [ ] Wiki validated
- [ ] Project state checked
- [ ] Staged changes reviewed (git status + git diff --staged)
- [ ] Conventional commit message generated (type(scope): description)
- [ ] Message follows format rules (lowercase title, max 50 chars, body explains why)
- [ ] No ads or auto-attribution lines
- [ ] User approved

## Red Flags

- Committing with wiki validation errors
- Pushing without user confirmation
- Using vague commit messages without scope
- Including unintended files (node_modules, secrets, build artifacts)
- Committing unrelated changes in one commit
- Not checking staged diff before committing


## Final Response Contract

All built-in skills in scope must end with the same user-facing information order: `wm-init`, `wm-spec`, `wm-plan`, `wm-research`, `wm-implement`, `wm-verify`, `wm-doc`, `wm-template`, `wm-extract`, and `wm-commit`.

Required order for the final user-facing response:

1. Goal/result - state what was accomplished.
2. Key details - include the most important supporting context, refs, assumptions, or validation.
3. Next action - recommend a concrete follow-up command only when a natural handoff exists.

Keep this concise for CLI use. Skill-specific content may extend the key-details section, but must not replace or reorder the shared structure.

Out of scope: explaining, syncing, or generating `.claude/skills/*`. Runtime auto-sync already handles platform copies, so this skill source only defines the built-in output contract.

For `wm-commit`, the key details should cover:
- files changed and scope of commit
- verification results
- task or feature area referenced

## Related Skills

- `/wm-extract` — Extract patterns or decisions from the work
- `/wm-plan <id>` — Continue with next task

## Next Step Suggestion

After commit:

```
/wm-extract   — Extract patterns or decisions from the work
/wm-spec      — Start next spec
/wm-go        — Continue with next pipeline
```
