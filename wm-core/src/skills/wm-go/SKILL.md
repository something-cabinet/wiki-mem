---
name: wm-go
description: Execute entire spec pipeline — plan, implement, verify — without review gates
---

# Go Mode

**Announce:** "Using wm-go for spec [name]."

**Core principle:** SPEC APPROVED → GENERATE TASKS → IMPLEMENT ALL → VERIFY → COMMIT.

## Process

1. **Validate spec** — check approved tag, ACs exist
2. **Generate tasks** — create tasks from requirements with fulfills mapping
3. **Plan + implement each** — loop through tasks, plan directly, implement, check ACs
4. **Full verification** — run `wm_validate.check({ "scope": "sdd" })`
5. **Commit** — stage all, generate conventional commit message, ask user

## Re-run Behavior

Already-done tasks are skipped. Continues from where it left off.

## Error Handling

Build/test failures: fix and re-run. Unfixable: mark task blocked and continue.
