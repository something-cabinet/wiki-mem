---
title: gh-commit
description: Verify wiki health and prepare a commit after implementation.
---

## Steps

### Step 1: Validate the wiki
Call `wm_validate.check` to verify all pages have required fields.

### Step 2: Lint check
Call `wm_lint.check` to find orphans, broken refs, and missing acceptance criteria.

### Step 3: Fix issues
Run `wm_lint.fix` to auto-fix common issues (missing titles, types).

### Step 4: Generate commit message
Summarize changes referencing wiki pages created or modified.
