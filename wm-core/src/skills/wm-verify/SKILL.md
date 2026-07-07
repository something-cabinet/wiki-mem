---
name: wm-verify
description: Run SDD verification and coverage reporting
---

# SDD Verification

**Announce:** "Using wm-verify for [spec or all]."

**Core principle:** EVERY SPEC REQUIREMENT → TRACKED → TESTED → VERIFIED.

## Step 1: Run SDD Validation

```json
wm_validate.check({ "scope": "sdd" })
```

## Step 2: Review Coverage

Check:
- All tasks complete for each spec
- All acceptance criteria checked
- No broken references
- No orphan pages

## Step 3: Report

```
SDD Coverage Report
═══════════════════
Spec: specs/<name>
Tasks: X/X complete
ACs: Y/Z verified
```
