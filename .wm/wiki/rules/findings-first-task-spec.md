---
title: Findings must create task + spec before implementation
type: rule
status: active
category: workflow
rationale: "Skipping task/spec creation leads to ad-hoc fixes without proper planning, acceptance criteria, traceability, or review."
---

## Rule: Create a task and spec for every finding before implementing

Any finding from reviews, audits, or analysis **must** have a wiki task page and a spec page created first, before any code changes are made.

### Why

- Ensures changes are planned and approved before execution
- Provides traceability — each fix links back to a task ID
- Captures acceptance criteria upfront so the result can be verified
- Prevents scope creep from unplanned findings

### How

1. Create a **task** page (type: `task`) with:
   - Title describing the finding
   - Severity (High / Medium / Low)
   - Acceptance criteria (checklist format)
   - References to affected files

2. Create a **spec** page (type: `spec`) with:
   - Approach and design for the fix
   - Files to change and how
   - Anything the implementer needs to know

3. Only after both pages exist and the task is formally started should implementation begin.

4. Reference the task ID in the commit message and any related work.

### Exceptions

- Trivial typos or one-line fixes in documentation only (not code)
- Emergency hotfixes where the pipeline is broken — but create the task+spec immediately after

### Example

```markdown
# Task: Fix overscaling hover on nav items

Severity: Medium

AC:
- [ ] Remove scale transform from nav item hover
- [ ] Use subtle background shift instead
- [ ] Ensure all states (hover, focus, active) are covered

Files:
- apps/wm-web/src/app/components/sidebar/nav-item.component.ts
```
