---
name: wm-verify
description: Run SDD verification and coverage reporting across all specs and tasks
---

# SDD Verification

**Announce:** "Using wm-verify for [spec or all]."

**Core principle:** VERIFY SPEC COVERAGE → REPORT WARNINGS → SUGGEST FIXES.

## Inputs

- Optional: specific spec path to verify (default: all specs)
- Current project state with tasks and specs

## Step 1: Run SDD Validation

```json
wm_validate.check({"scope": "sdd"})
```

## Step 2: Review Coverage

Check each spec:

- All tasks complete
- All acceptance criteria checked
- No broken references
- No orphan pages (pages not linked from any other page)
- All must-fix findings from review resolved

## Step 3: Check Index Status

```json
wm_validate.check({})  # general health check
```

## Step 4: Analyze Results

**Good coverage (>80%):**
> SDD coverage is healthy. All tasks are properly linked to specs.

**Medium coverage (50-80%):**
> Some tasks are missing spec references. Consider:
> - Link existing tasks to specs: `wm_task.update <id> --spec specs/<name>`
> - Create specs for unlinked work: `/wm-spec <feature-name>`

**Low coverage (<50%):**
> Many tasks lack spec references. For better traceability:
> 1. Create specs for major features: `/wm-spec <feature>`
> 2. Link tasks to specs: `wm_task.update <id> --spec specs/<name>`
> 3. Use `/wm-plan --from @page/specs/<name>` for new tasks

## Step 5: Suggest Actions

Based on warnings, add the most relevant fixes:

**For tasks without spec:**
> Link task to spec:
> ```json
> wm_task.update({"id": "<id>"})
> ```

**For incomplete ACs:**
> Check task progress:
> ```bash
> wm_task.get ... --plain
> ```

**For approved specs without tasks:**
> Create tasks from spec:
> ```
> /wm-plan --from @page/specs/<name>
> ```

## Step 6: Report

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

### Issues Found

```markdown
- Spec `specs/<name>`: Task task-xxx is still in-progress
- Spec `specs/<name>`: AC-3 not checked in any task
- Broken ref: @page/missing-page referenced but not found
```

## Entity-Specific Validation (Optional)

To validate a single task or doc (saves tokens):

```json
// Validate single task
wm_validate.check({"entity": "abc123"})

// Validate single doc
wm_validate.check({"entity": "specs/user-auth"})
```

## Checklist

- [ ] SDD validation run
- [ ] All specs checked
- [ ] Task completion verified per spec
- [ ] AC coverage verified per spec
- [ ] No broken refs
- [ ] No orphan pages
- [ ] Coverage analyzed (>80% healthy, 50-80% warning, <50% action needed)
- [ ] Specific fix suggestions provided for issues
- [ ] Report generated

## Red Flags

- Running verification before any tasks are implemented
- Ignoring broken refs — each one is a gap in the spec-to-implementation chain
- Not checking orphan pages — valuable context may be lost
- Marking spec complete with unverified ACs
- Ignoring coverage warnings without suggesting fixes
- Claiming coverage is healthy without showing evidence

## Final Response Contract

All built-in skills in scope must end with the same user-facing information order: `wm-init`, `wm-spec`, `wm-plan`, `wm-research`, `wm-implement`, `wm-verify`, `wm-doc`, `wm-template`, `wm-extract`, and `wm-commit`.

Required order for the final user-facing response:

1. Goal/result - state what was accomplished.
2. Key details - include the most important supporting context, refs, assumptions, or validation.
3. Next action - recommend a concrete follow-up command only when a natural handoff exists.

Keep this concise for CLI use. Skill-specific content may extend the key-details section, but must not replace or reorder the shared structure.

Out of scope: explaining, syncing, or generating `.claude/skills/*`. Runtime auto-sync already handles platform copies, so this skill source only defines the built-in output contract.

For `wm-verify`, the key details should cover:
- verification scope, pass/fail status, coverage gaps found

## Related Skills

- `/wm-implement <task-id>` — Complete remaining tasks
- `/wm-commit` — Commit verified changes
- `/wm-flow @page/<spec>` — Continue full pipeline


## Next Step Suggestion

```
/wm-implement <task-id>   — Complete remaining tasks
/wm-commit                 — Commit verified changes
/wm-go @page/<spec>       — Continue full pipeline
```
