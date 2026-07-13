---
name: wm-validate
description: Validate wiki health — check broken refs, page completeness, orphan pages, and consolidate learnings
---

# Wiki Validation

**Announce:** "Using wm-validate to check wiki health."

**Core principle:** VALIDATE FIRST, FIX SECOND, REBUILD THIRD.

## Inputs

- Optional: specific page ID to validate (default: entire wiki)
- Optional: `--fix` to auto-resolve simple issues
- Optional: `--scope sdd` for SDD validation (spec coverage)

## Step 1: Run Validation

```json
wm_validate.check({})
```

Review the output:
- `errors` — broken refs, missing fields (must fix)
- `warnings` — orphan pages, missing optional fields (should fix)
- `status` — "pass" or "fail"

### Error Types

| Error | Severity | Meaning |
|-------|----------|---------|
| Broken wiki ref | 🔴 Error | `relates_to` target page doesn't exist |
| Empty title | 🔴 Error | Page has no title |
| Missing ACs | 🟡 Warning | Task page has no acceptance criteria |
| No assignee | 🟡 Warning | Task has no assignee |
| Orphan page | 🟡 Warning | No incoming links from other pages |
| Missing spec fields | 🟡 Warning | Spec missing stakeholders |
| Missing decision fields | 🟡 Warning | Decision missing context/options/rationale |

## Step 2: Fix Broken References

For each broken `wiki:` ref error, determine the right action:

1. **Missing page** — the referenced page should exist. Create it:
   ```json
    wm_doc.create({"path": "<folder>/<page-slug>", "title": "...", "tags": ["<type>"]})
   ```

2. **Wrong reference** — the reference points to a fictional/example page. Remove it from the source page's `relates_to`:
   ```json
   wm_doc.update({"path": "<page-id>", "tags": [... correct tags ...]})
   ```

3. **Format mismatch** — `wiki:concepts:slug` format vs `concepts/slug` ID. Normalize to match actual page IDs.

## Step 3: Consolidate Learnings

If validation reveals repeated issues or patterns worth capturing:

1. Search existing memory to avoid duplicates:
   ```json
   wm_memory.list({"tag": "validation"})
   ```

2. Create a learning page if a pattern emerges:
   ```json
    wm_doc.create({"path": "learnings/validation-<topic-slug>", "title": "<Learning: Topic>",
  "tags": ["learning"],
  "content": "## Signal\n\n...\n\n## Fix\n\n..."})
   ```

3. Alternatively, save as quick memory:
   ```json
wm_memory.add({"id": "validation-<slug>", "title": "Validation pattern: <summary>",
  "content": "<2-3 sentence summary>",
  "category": "pattern",
  "tags": ["validation"]})
   ```

## Step 4: SDD Verification (if spec-linked)

If `--scope sdd` is passed, run SDD validation:

```json
wm_validate.check({"scope": "sdd"})
```

This checks spec ACs, task status, and spec-to-task coverage.

## Step 5: Index Note

Currently no explicit index-rebuild tool is exposed. The search index stays current through the backend's own sync cycle.

## Checklist

- [ ] Validation run — errors and warnings reviewed
- [ ] Broken refs fixed (pages created or refs removed)
- [ ] Orphan pages reviewed (linked or acknowledged)
- [ ] Learnings consolidated if patterns emerged
- [ ] SDD verification run if spec-linked
- [ ] Changes committed

## Red Flags

- Skipping error fixes — broken refs compound over time
- Forgetting to re-validate after fixes
- Fixing refs without checking if the target page should exist
- Not consolidating patterns — repeated issues indicate a knowledge gap

## Next Step Suggestion

```
/wm-commit             — Commit fixes
/wm-plan <task-id>     — Continue with next task
```
