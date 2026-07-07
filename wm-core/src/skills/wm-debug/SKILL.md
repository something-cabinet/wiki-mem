---
name: wm-debug
description: Structured debugging — triage, reproduce, diagnose, fix, learn
---

# Debugging

**Announce:** "Using wm-debug for [error/issue]."

**Core principle:** CLASSIFY → REPRODUCE → ROOT CAUSE → FIX → LEARN.

## Step 1: Classify

| Type | Signals |
|------|---------|
| Build failure | Compilation, type error, missing dep |
| Test failure | Assertion mismatch, timeout, flaky |
| Runtime error | Crash, panic, uncaught exception |
| Integration failure | HTTP error, env missing, API mismatch |

## Step 2: Check Known Patterns

```json
wm_search.query({ "query": "<error pattern>", "type": "doc", "tag": "learning" })
wm_search.query({ "query": "<error pattern>", "type": "memory" })
```

## Step 3: Reproduce & Diagnose

Run failing command verbatim. Read implicated files. Check recent changes.

## Step 4: Fix

Implement fix. Verify by re-running original failing command. Run broader checks.

## Step 5: Learn

If new pattern (≥15min save): capture as memory or learning doc.
