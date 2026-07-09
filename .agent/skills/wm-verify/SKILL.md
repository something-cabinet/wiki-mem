---
name: wm-verify
description: Run SDD verification and coverage reporting across all specs and tasks
---

# SDD Verification

**Announce:** "Using wm-verify for [spec or all]."

**Core principle:** EVERY SPEC REQUIREMENT → TRACKED → TESTED → VERIFIED.

## Inputs

- Optional: specific spec path to verify (default: all specs)
- Current project state with tasks and specs

## Step 1: Run SDD Validation

```json
wm:validate.check({ "scope": "sdd" })
```

## Step 2: Review Coverage

Check each spec:

- All tasks complete
- All acceptance criteria checked
- No broken references
- No orphan pages (pages not linked from any other page)
- All must-fix (P0) findings from review resolved

## Step 3: Check Index Status

```json
wm:index.status({})
```

Ensure the search index is current and reflects all recent changes.

## Step 4: Report

```markdown
## SDD Coverage Report
═══════════════════════════════════════
### Spec: specs/<name>
- **Tasks:** X/X complete
- **ACs:** Y/Z verified
- **Refs:** All valid
- **Status:** ✅ Complete / ⚠️ Partial / ❌ Issues

### Spec: specs/<other-name>
- **Tasks:** X/X complete
- **ACs:** Y/Z verified
- **Refs:** All valid
- **Status:** ✅ Complete / ⚠️ Partial / ❌ Issues
```

## Step 5: Report Issues

If any issues found, list them:

```markdown
### Issues Found
- Spec `specs/<name>`: Task task-xxx is still in-progress
- Spec `specs/<name>`: AC-3 not checked in any task
- Broken ref: @page/missing-page referenced but not found
```

## Checklist

- [ ] SDD validation run
- [ ] All specs checked
- [ ] Task completion verified per spec
- [ ] AC coverage verified per spec
- [ ] No broken refs
- [ ] No orphan pages
- [ ] Index status confirmed
- [ ] Report generated

## Red Flags

- Running verification before any tasks are implemented
- Ignoring broken refs — each one is a gap in the spec-to-implementation chain
- Not checking orphan pages — valuable context may be lost
- Marking spec complete with unverified ACs

## Next Step Suggestion

```
/wm-implement <task-id>   — Complete remaining tasks
/wm-commit                 — Commit verified changes
/wm-go @page/<spec>       — Continue full pipeline
```
