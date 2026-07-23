---
name: wm-debug
description: Structured debugging — triage, reproduce, diagnose, fix, and capture learnings
---

# Debugging

**Announce:** "Using wm-debug for [error/issue]."

**Core principle:** CLASSIFY → REPRODUCE → ROOT CAUSE → FIX → LEARN.

## Inputs

- Error message, stack trace, or behavioral description
- Optional: task ID or context of the failing feature

## Step 1: Triage — Classify the Issue

Classify before investigating. Misclassifying wastes time.

| Type | Signals |
|------|---------|
| **Build failure** | Compilation error, type error, missing module, bundler failure |
| **Test failure** | Assertion mismatch, snapshot diff, timeout, flaky intermittent |
| **Runtime error** | Crash, uncaught exception, undefined behavior |
| **Integration failure** | HTTP 4xx/5xx, env variable missing, API schema mismatch |
| **Blocked task** | Circular dependency, conflicting changes, unclear requirement |

**Output:** One-line classification: `[TYPE] in [component]: [symptom]`

## Step 2: Search Known Patterns

Check if this issue has been seen before:

```json
wm_search.query({"q": "<error message>", "type": "doc"})
wm_search.query({"q": "<error message>", "type": "memory"})
wm_search.query({"q": "<error pattern>", "type": "doc"})
```

If a known pattern matches → jump to Step 5 (Fix) using the documented resolution.

## Step 3: Reproduce & Diagnose

### 3a. Reproduce

Run the exact failing command verbatim:

```bash
# Whatever failed — run it exactly
<failing-command> 2>&1
```

Capture error output verbatim. Exact line numbers and messages matter.

Run twice — if intermittent, classify as flaky (check shared state, race conditions, test ordering).

### 3b. Read implicated files

Read exactly the files mentioned in the error output. Do not read the entire codebase.

### 3c. Check recent changes

```bash
git log --oneline -10
git diff HEAD~3 -- <failing-file>
```

If a recent commit introduced the failure → fix is likely adjusting that change.

### 3d. Check task context (if task ID provided)

```json
wm_task.get({"id": "$ARGUMENTS"})
```

Does the failure indicate the task was implemented against the wrong spec, or correctly but the spec was wrong?

### 3e. Narrow to root cause

Write a **one-sentence root cause**:

> Root cause: `<file>:<line>` — `<what is wrong and why>`

If you cannot write this sentence, you do not have the root cause yet. **Do NOT proceed to Fix.**

## Step 4: Diagnose with Tools

```json
wm_project.status()
wm_doc.get({"action": "get", "id": "wiki:<relevant-doc>"})
wm_graph.neighbors({"id": "<affected-module>"})
```

Isolate the root cause:
1. What is the expected behavior?
2. What is the actual behavior?
3. What changed between working and broken?
4. What assumptions were violated?

## Step 5: Fix

### Small fix (1–3 files, obvious change)

- Implement directly
- Run verification immediately:

```bash
# Re-run the originally failing command
<failing-command>
```

### Substantial fix (cross-cutting, logic redesign)

- If within a task, append notes about the issue:

```json
wm_task.update({"id": "<id>"})
```

- If standalone, consider creating a task:

```json
wm_task.create({"id": "fix-<root-cause-slug>", "title": "Fix: <root cause summary>", "content": "Root cause: <detail>\nFix approach: <approach>",
  "priority": "high", "tags": ["bugfix"], "acceptance_criteria": ["Verify fix resolves <root cause>", "Test passes"]})
```

### Verify the fix

Run the exact command that originally failed. It must pass cleanly:

```bash
<original-failing-command>
```

Also run broader checks for regressions:

```bash
# Project-specific build/test/lint
```

If verification fails → return to Step 3 with new information. Do NOT report success.

## Step 6: Validate

```json
wm_validate.check({})
```

## Step 7: Capture the Learning

Ask: would this save ≥15 minutes if a future agent knew it?

### Quick pattern (<5 min to describe) — save to memory:

```json
wm_memory.add({"id": "debug-<error-slug>", "title": "Debug: <error pattern>",
  "content": "Root cause: <sentence>. Fix: <what resolves it>",
  "layer": "project",
  "tags": ["debug", "<domain>"]})
```

### Detailed pattern (worth a full writeup) — create or update a learning doc:

Search for existing learning doc:
```json
wm_search.query({"q": "<failure domain>", "type": "doc"})
```

**If existing learning doc found — update it:**
```json
wm_doc.get({"action": "get", "id": "wiki:<existing-path>"})
# Then update with full content (WM has no appendContent — read, modify, write):
wm_doc({"action": "update", "id": "wiki:<existing-path>",
  "content": "<existing-full-content>\n\n## <Date> — <Classification>\n\n**Root cause:** <sentence>\n**Signal:** <how to recognize>\n**Fix:** <what resolves it>"})
```

**If no existing doc — create new:**
```json
wm_doc({"action": "create", "path": "patterns/<domain>-<pattern-slug>", "title": "Learning: <domain> — <pattern>",
  "tags": ["learning", "<domain>"],
  "content": "## Problem\n\n<what goes wrong>\n\n## Root Cause\n\n<why it happens>\n\n## Signal\n\n<how to recognize this pattern>\n\n## Fix\n\n<what resolves it>\n\n## Source\n\n@task-<id> (if applicable)"})
```

### Known pattern that didn't work?

If the documented resolution failed or is outdated:

```json
wm_doc.get({"action": "get", "id": "wiki:<learning-path>"})
# Then rewrite with appended content (WM has no appendContent — read, modify, write):
wm_doc({"action": "update", "id": "wiki:<learning-path>",
  "content": "<existing-full-content>\n\n⚠️ **Update <date>:** Resolution no longer accurate — <what changed>"})
```

## Quick Reference

| Situation | First action |
|-----------|-------------|
| Build fails | `git log --oneline -10` — check recent changes |
| Test fails | Run test verbatim, capture exact assertion output |
| Flaky test | Run 5× — if intermittent, check shared state/ordering |
| Runtime crash | Read stack trace top-to-bottom, find first line in your code |
| Integration error | Check env vars, then API response body (not just status code) |
| Recurring issue | Search learnings docs first |

## Checklist

- [ ] Issue classified with one-line summary
- [ ] Known patterns searched
- [ ] Issue reproduced with exact command
- [ ] Root cause identified (one sentence — if you can't write it, you don't have it)
- [ ] Fix implemented and verified (re-run failing command)
- [ ] Validated
- [ ] Learning captured if valuable (memory or doc)
- [ ] Timer stopped

## Red Flags

- Fixing symptoms without root cause
- Skipping reproduction — "I think I know the cause" is not enough
- Not searching existing learnings — someone may have solved this before
- Capturing vague learnings that won't help in the future
- Applying fix without verification
- Committing fix without running verification
- Not capturing a learning when the fix took >15 minutes to find

## Final Response Contract

All built-in skills in scope must end with the same user-facing information order: `wm-init`, `wm-spec`, `wm-plan`, `wm-research`, `wm-implement`, `wm-verify`, `wm-doc`, `wm-template`, `wm-extract`, and `wm-commit`.

Required order for the final user-facing response:

1. Goal/result - state what was accomplished.
2. Key details - include the most important supporting context, refs, assumptions, or validation.
3. Next action - recommend a concrete follow-up command only when a natural handoff exists.

Keep this concise for CLI use. Skill-specific content may extend the key-details section, but must not replace or reorder the shared structure.

Out of scope: explaining, syncing, or generating `.claude/skills/*`. Runtime auto-sync already handles platform copies, so this skill source only defines the built-in output contract.

For `wm-debug`, the key details should cover:
- root cause found
- fix applied
- tests or verification run

## Related Skills

- `/wm-extract` — Extract the debug pattern as a formal learning
- `/wm-plan <id>` — Plan remaining work affected by the fix
- `/wm-commit` — Commit the fix

## Next Step Suggestion

```
/wm-extract   — Extract the debug pattern as a formal learning
/wm-plan      — Plan remaining work affected by the fix
/wm-commit    — Commit the fix
```
