---
name: wm-verify
description: Run SDD verification and coverage reporting
---

# Verify

**Announce:** "Using wm-verify."

## Steps

### 1. Validate graph
```
wm_validate.check()
```

### 2. Report coverage
Check task pages:
- Do they have acceptance criteria?
- Are ACs checked off?
- Are they linked to specs via `implements` edges?

### 3. Check for gaps
- Orphan pages (no inbound edges) — add relationships
- Broken relates_to refs — fix target IDs
- Stale sources — re-process if source content changed
