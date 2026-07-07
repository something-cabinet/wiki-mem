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

## Step 1: Classify the Issue

| Type | Signals |
|------|---------|
| Build failure | Compilation, type error, missing dependency |
| Test failure | Assertion mismatch, timeout, flaky test |
| Runtime error | Crash, panic, uncaught exception |
| Integration failure | HTTP error, env missing, API mismatch |

## Step 2: Search Known Patterns

Check if this issue has been seen before:

```json
wm_search.query({ "query": "<error message>", "type": "page", "tag": "learning" })
wm_search.query({ "query": "<error message>", "type": "memory" })
wm_search.query({ "query": "<error pattern>", "type": "page" })
```

## Step 3: Check Recent Changes

```json
wm_log.recent({ "limit": 20 })
wm_log.filter({ "level": "error", "limit": 20 })
```

Review recent changes and error logs to identify what might have introduced the issue.

## Step 4: Reproduce

Run the failing command verbatim. Capture:
- Exact command and arguments
- Full error output
- Environment context

```json
wm_project.status({})
wm_model.status({})
```

## Step 5: Diagnose

Read implicated files. Trace the execution path:

```json
wm_page.get({ "id": "<relevant-doc>", "smart": true })
wm_graph.neighbors({ "id": "<affected-module>" })
```

Isolate the root cause:
1. What is the expected behavior?
2. What is the actual behavior?
3. What changed between working and broken?
4. What assumptions were violated?

## Step 6: Fix

Implement the fix:

```json
wm_time.start({ "taskId": "<debug-task>" })
```

After fixing, verify:
- Run the original failing command — it should pass
- Run broader related tests
- Run lint checks:

```json
wm_lint.check({})
```

## Step 7: Validate

```json
wm_validate.check({})
```

## Step 8: Capture the Learning

If this is a new pattern (≥15 min save for future agents):

```json
wm_page.create({
  "id": "learnings/debug/<topic-slug>",
  "title": "Debug: <error pattern>",
  "tags": ["learning", "debug"],
  "content": "## Problem\n\n[Error]\n\n## Root Cause\n\n...\n\n## Signal\n\n...\n\n## Fix\n\n..."
})
```

Or add a quick memory page:

```json
wm_page.create({
  "id": "memories/debug-<topic-slug>",
  "title": "Debug: <error pattern>",
  "tags": ["debug", "learning", "<domain>"],
  "content": "<2-3 sentence summary of fix>"
})
```

## Checklist

- [ ] Issue classified by type
- [ ] Known patterns searched
- [ ] Recent changes and logs reviewed
- [ ] Issue reproduced with exact command
- [ ] Root cause identified
- [ ] Fix implemented and verified
- [ ] Validated
- [ ] Learning captured if valuable
- [ ] Timer stopped

## Red Flags

- Skipping reproduction — "I think I know the cause" is not enough
- Fixing without understanding root cause
- Not searching existing learnings — someone may have solved this before
- Capturing vague learnings that won't help in the future
- Applying fix without verification

## Next Step Suggestion

```
/wm-extract   — Extract the debug pattern as a formal learning
/wm-plan      — Plan remaining work affected by the fix
/wm-commit    — Commit the fix
```
