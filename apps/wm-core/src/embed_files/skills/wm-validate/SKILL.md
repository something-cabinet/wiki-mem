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
    wm_doc({"action": "create", "path": "<folder>/<page-slug>", "title": "...", "tags": ["<type>"]})
   ```

2. **Wrong reference** — the reference points to a fictional/example page. Remove it from the source page's `relates_to`:
   ```json
   wm_doc({"action": "update", "id": "wiki:<page-id>", "tags": [... correct tags ...]})
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
    wm_doc({"action": "create", "path": "concepts/validation-<topic-slug>", "title": "<Learning: Topic>",
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

### Review Per-Spec Coverage

Check each spec:

- All tasks complete
- All acceptance criteria checked
- No broken references
- No orphan pages (pages not linked from any other page)

### Analyze Coverage Thresholds

**Good coverage (>=80%):** 🟢 Healthy — automatic proceed.
> SDD coverage is healthy. All tasks are properly linked to specs.

**Medium coverage (50-80%):** 🟡 Warning — proceed with caution.
> Some tasks are missing spec references. Consider:
> - Link existing tasks to specs: `wm_task.update <id> --spec specs/<name>`
> - Create specs for unlinked work: `/wm-spec <feature-name>`

**Low coverage (<50%):** 🔴 Action required before continue.
> Many tasks lack spec references. For better traceability:
> 1. Create specs for major features: `/wm-spec <feature>`
> 2. Link tasks to specs: `wm_task.update <id> --spec specs/<name>`
> 3. Use `/wm-plan --from @page/specs/<name>` for new tasks

### Coverage Report

```markdown
## SDD Coverage Report
═══════════════════════════════════════
### Spec: specs/<name>
- **Tasks:** X/X complete
- **ACs:** Y/Z verified
- **Refs:** All valid
- **Status:** 🟢 Complete / 🟡 Partial / 🔴 Issues

### Spec: specs/<other-name>
- **Tasks:** X/X complete
- **ACs:** Y/Z verified
- **Refs:** All valid
- **Status:** 🟢 Complete / 🟡 Partial / 🔴 Issues
```

### Issues Found

```markdown
- Spec `specs/<name>`: Task task-xxx is still in-progress
- Spec `specs/<name>`: AC-3 not checked in any task
- Broken ref: @page/missing-page referenced but not found
```

### Suggest Fixes

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

### Entity-Specific Validation (Optional)

To validate a single task or doc (saves tokens):

```json
// Validate single task
wm_validate.check({"entity": "abc123"})

// Validate single doc
wm_validate.check({"entity": "specs/user-auth"})
```

## Step 5: Index Note

Currently no explicit index-rebuild tool is exposed. The search index stays current through the backend's own sync cycle.

## Checklist

- [ ] Validation run — errors and warnings reviewed
- [ ] Broken refs fixed (pages created or refs removed)
- [ ] Orphan pages reviewed (linked or acknowledged)
- [ ] Learnings consolidated if patterns emerged
- [ ] SDD verification run if spec-linked
- [ ] All specs checked
- [ ] Task completion verified per spec
- [ ] AC coverage verified per spec
- [ ] Coverage analyzed — thresholds applied and reported
- [ ] Specific fix suggestions provided for issues
- [ ] Changes committed

## Red Flags

- Skipping error fixes — broken refs compound over time
- Forgetting to re-validate after fixes
- Fixing refs without checking if the target page should exist
- Not consolidating patterns — repeated issues indicate a knowledge gap

## Final Response Contract

All built-in skills in scope must end with the same user-facing information order: `wm-init`, `wm-spec`, `wm-plan`, `wm-research`, `wm-implement`, `wm-verify`, `wm-doc`, `wm-template`, `wm-extract`, and `wm-commit`.

Required order for the final user-facing response:

1. Goal/result - state what was accomplished.
2. Key details - include the most important supporting context, refs, assumptions, or validation.
3. Next action - recommend a concrete follow-up command only when a natural handoff exists.

Keep this concise for CLI use. Skill-specific content may extend the key-details section, but must not replace or reorder the shared structure.

Out of scope: explaining, syncing, or generating `.claude/skills/*`. Runtime auto-sync already handles platform copies, so this skill source only defines the built-in output contract.

For `wm-validate`, the key details should cover:
- validation scope, errors found, warnings, fixes applied

## Related Skills

- `/wm-commit` — Commit fixes
- `/wm-plan <task-id>` — Continue with next task
- `/wm-flow @page/<spec>` — Resume spec pipeline


## Next Step Suggestion

```
/wm-commit             — Commit fixes
/wm-plan <task-id>     — Continue with next task
```
