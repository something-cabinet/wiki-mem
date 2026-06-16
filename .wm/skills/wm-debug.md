---
name: wm-debug
description: Debug errors and failures with wiki-backed memory and structured triage
---

# Debug

**Announce:** "Using wm-debug."

**Core principle:** STOP → READ → ANALYZE → FIX → LEARN.

## Steps

### 1. Read the error
Capture the exact error message and relevant context.

### 2. Search wiki for similar issues
```
wm_search.query(q="<error message>", mode=keyword, limit=10)
wm_search.query(q="<component> failures", mode=hybrid, limit=10)
```

### 3. Check existing decisions
```
wm_graph.neighbors(id="wiki:decisions:<component>")
```
Follow `causes` and `mitigates` edges to find root causes and fixes.

### 4. Fix the issue
Apply the fix. If you hit a second failure, stop and escalate.

### 5. Extract the learning
```
wm-extract <task-id>
```
Save what you learned about debugging this issue so future sessions benefit.
