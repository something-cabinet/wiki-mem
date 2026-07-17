---
name: wm-extract
description: Extract reusable patterns, decisions, and failures into wiki pages
---

# Extracting Knowledge (with Compounding)

**Announce:** "Using wm-extract for [pattern/decision]."

**Core principle:** IF IT COST TIME TO LEARN, SAVE IT FOR LATER.
**Typed pages:** Each extraction goes into a properly typed page (pattern, decision, concept) with a `references` edge back to the source task. No more flat learnings docs.

## Inputs

- Usually a completed task ID
- Sometimes a code change, repeated pattern, or recurring support issue
- Optional: `--compound` flag for full 3-category analysis
- Optional: `--consolidate` flag to review and consolidate all existing learnings

## Mode Detection

Check `$ARGUMENTS`:
- Contains `--consolidate` → Go to "Consolidation Mode" section
- Otherwise → Continue with normal extraction flow

## Extraction Rules

- Extract patterns, decisions, AND failures — not just code patterns
- Prefer updating an existing doc over creating a duplicate
- Link the extracted knowledge back to the source task or source doc
- Only create a template if the pattern is genuinely reusable for generation
- Promote critical patterns to `patterns/critical-patterns` for future `wm-init` sessions

## Wiki Page Type Mapping

| Extraction Type | Wiki Subdirectory | PageType |
|----------------|-------------------|----------|
| Reusable pattern | `patterns/` | Pattern |
| Architecture decision (ADR) | `decisions/` | Decision |
| Domain concept | `concepts/` | Concept |
| Step-by-step guide | `howto/` | Howto |
| Reference / API doc | `reference/` | Reference |
| Failure / debugging log | `concepts/` | Concept |
| Informal note | `concepts/` (with `type: note`) | Note |

Wiki pages are stored under `.wm/wiki/` and accessible via `wm_page.get({"path": "<subdir>/<slug>"})`.

## Step 1: Review Source Material

```json
wm_task.get({"id": "$ARGUMENTS"})
# No log tool available; review git log instead
```

Review the task, recent logs, and changes to identify what is worth capturing.

Look for three categories:

| Category | What to look for |
|----------|-----------------|
| **Patterns** | Reusable code patterns, architecture approaches, integration techniques |
| **Decisions** | Good calls, bad calls, trade-offs, surprises |
| **Failures** | Bugs, wrong assumptions, wasted effort, missing prerequisites |

## Step 2: Check for Duplicates

Search existing wiki pages and WM memory to avoid duplicating knowledge:

```json
wm_search.query({"q": "<topic>", "type": "all", "mode": "keyword"})
wm_memory.list({"category": "pattern", "tag": "<domain>"})
```

If the topic already exists, skip or update instead of creating a duplicate.

## Step 3: Three-Category Analysis

### 3a. Patterns

Identify reusable patterns:
- Code patterns: new utilities, abstractions worth standardizing
- Architecture patterns: structural decisions that worked
- Process patterns: workflow approaches that saved time

### 3b. Decisions

Identify significant decisions:
- **GOOD_CALL**: decisions that proved correct or saved time
- **BAD_CALL**: decisions that required rework
- **SURPRISE**: things that turned out differently than expected
- **TRADEOFF**: conscious choices where alternatives were considered

### 3c. Failures

Identify failures and wasted effort:
- Bugs introduced and root causes
- Wrong assumptions that required backtracking
- Missing prerequisites discovered mid-execution
- Test gaps that allowed regressions

## Step 4: Create/Update Documentation

Each finding gets its own **typed page** with a `references` edge back to the source task. Use `wm_page.create` with the appropriate `type` parameter (see the page-type mapping table above).

### For patterns → Pattern Page

Create a typed pattern page with ADR-compatible field support:

```json
wm_page.create({"action": "create", "path": "patterns/<name-slug>", "title": "Pattern: <Name>",
  "type": "pattern", "tags": ["pattern", "<domain>"],
  "content": "<markdown content>"})
wm_page.link({"id": "wiki:patterns/<name-slug>", "target": "wiki:tasks/<source-task-id>", "edge_type": "references"})
```

#### Pattern Template

```markdown
## Problem
What problem does this pattern solve?

## Solution
The reusable approach or implementation.

## When to Use
Signals that this pattern applies.

## When Not to Use
Contexts where this pattern adds unnecessary complexity.

## Related
- @task-<id>
```

### For decisions → Decision Page

Architectural decisions become their own Decision page with ADR frontmatter:

```json
wm_page.create({"action": "create", "path": "decisions/<name-slug>", "title": "Decision: <Title>",
  "type": "decision", "status": "approved",
  "content": "<decision content with context/rationale/outcome>"})
wm_page.link({"id": "wiki:decisions/<name-slug>", "target": "wiki:tasks/<source-task-id>", "edge_type": "references"})
```

#### Decision Template

```markdown
## Context
What situation led to this decision?

## Decision
What was chosen.

## Rationale
Why this option over alternatives (trade-offs considered).

## Consequences
What this decision means for future work.

## Related
- @task-<id>
```

### For failures → Concept Page

Failures and debugging stories become Concept pages (they don't need per-type data):

```json
wm_page.create({"action": "create", "path": "concepts/<name-slug>", "title": "Failure: <Title>",
  "type": "concept", "tags": ["failure", "<domain>"],
  "content": "<markdown content>"})
wm_page.link({"id": "wiki:concepts/<name-slug>", "target": "wiki:tasks/<source-task-id>", "edge_type": "references"})
```

#### Failure Template

```markdown
## What went wrong
Description of the failure or bug.

## Root cause
The underlying cause (not just the symptom).

## Prevention
What to do differently next time.

## Time lost
Estimated wasted time.

## Related
- @task-<id>
```

### Edge type reference (extract-relevant subset):

| Usage | Edge | Direction |
|---|---|---|
| Extraction links back to source | `references` | Pattern/Decision/Concept → Task |
| Pattern is an example of a concept | `example_of` | Pattern → Concept |
| Decision supersedes a previous one | `supersedes` | Decision → Decision |
| Concept extends a parent concept | `extends` | Concept → Concept |
| Generic relation | `relates_to` | Page ↔ Page |

See @wiki/concepts/edge-types for the full 16-type reference.

## Step 5: Save Quick Memory (recall aid)

For each extracted pattern or decision worth quick recall, save a concise memory entry alongside the doc:

```json
wm_memory.add({"id": "<pattern-slug>", "title": "<pattern/decision name>",
  "content": "<2-3 sentence summary>. Full reference: @doc/<path>",
  "layer": "project",
  "tags": ["<domain>"]})
```

Memory = fast agent recall in future sessions. Doc = full structured reference.
Do NOT duplicate the entire doc content — store a summary and link to the doc.
Skip this step if the extraction produced nothing generalizable.

## Step 6: Create Template (if code-generatable)

```json
wm_template.create({"name": "<pattern-name>",
  "description": "Generate <what>",
  "content": "Template content with {{variable}} placeholders"})
```

## Step 7: Promote to Critical

If the knowledge meets ALL criteria:
- Affects more than one future feature
- Would cause **≥30 minutes** wasted effort if unknown
- Is generalizable, not implementation-specific

Check if critical-patterns page exists:

```json
wm_search.query({"q": "critical patterns", "type": "all"})
```

**If exists — append:**

```json
wm_page.get({"id": "wiki:patterns:critical-patterns"})
# WM has no appendContent — read, modify, then write full:
wm_page.update({"action": "update", "id": "wiki:patterns:critical-patterns",
  "content": "<existing-full-content>\n\n## [Date] <Learning Title>\n**Category:** pattern / decision / failure\n**Source:** @task-<id>\n**Tags:** [tag1, tag2]\n\n<2-4 sentence summary and what to do differently>\n\n**Full entry:** @wiki/patterns/<slug>"})
```

**If not exists — create:**

```json
wm_page.create({"action": "create", "path": "patterns/critical-patterns", "title": "Critical Patterns",
  "type": "pattern", "tags": ["critical"],
  "content": "# Critical Patterns\n\nPromoted learnings from completed work. Read this at the start of every session via `wm-init`. These are lessons that cost the most to learn and save the most by knowing.\n\n---"})
```

**Calibration:** Do NOT promote everything. If critical-patterns grows past 20-30 entries it becomes noise. Only promote learnings that would have saved ≥30 minutes if known in advance.

## Step 8: Validate

```json
wm_validate.check({"entity": "<doc-path>"})
```

If errors found, fix before continuing.

## Step 9: Link Back to Task

```json
wm_task.update({"id": "$ARGUMENTS"})
```

---

# Consolidation Mode (Dream Lite)

When `$ARGUMENTS` contains `--consolidate`:

**Announce:** "Using wm-extract --consolidate to review and consolidate learnings."

Scan all existing learnings docs, merge duplicates, flag outdated entries, and promote new critical patterns. Run on-demand when the learnings folder feels messy or after a batch of completed work.

## C-Step 1: Scan All Learnings

```json
wm_doc.list({"action": "list"})
```

Read each learning doc:

```json
wm_doc.get({"action": "get", "id": "wiki:<path>"})
```

## C-Step 2: Identify Issues

For each learning doc, check:

### Duplicates
- Two docs covering the same root cause or pattern
- Same advice phrased differently across docs
- → Merge into one, delete the other

### Outdated
- Fix/pattern references code that no longer exists
- Advice contradicts current architecture or conventions
- → Update or mark as outdated with date

### Missing Promotions
- Learning that meets critical criteria (affects multiple features, saves ≥30 min) but isn't in critical-patterns
- → Propose promotion

### Orphaned
- Learning that references a task or doc that no longer exists
- → Fix ref or note the context is lost

## C-Step 3: Apply Changes

For each issue found, present to user:

```
Consolidation findings:

1. MERGE: "Learning: auth token" + "Learning: JWT refresh" → same root cause
   → Merge into "Learning: auth token handling"?

2. OUTDATED: "Learning: webpack config" — references webpack.config.js which was removed
   → Mark outdated or delete?

3. PROMOTE: "Learning: Go test race conditions" — saved 2h on 3 separate tasks
   → Promote to critical-patterns?

4. ORPHAN: "Learning: SSE reconnect" — references @task-abc123 which doesn't exist
   → Keep content, remove broken ref?

Apply all? (yes / review each / skip)
```

**If "review each":** present one at a time, apply user's choice.
**If "yes":** apply all suggested changes.

## C-Step 4: Report

```
Consolidation complete:
- Merged: X docs
- Updated: X docs
- Promoted: X to critical-patterns
- Orphans fixed: X
- Total learnings: X docs
```

---

## No-Op Case

If the work is too specific to generalize, say so explicitly and do not force a new doc.

**Do NOT fabricate findings.** If the task ran smoothly with no surprises, write that. A short learning with 2 genuine entries is better than a long doc with invented ones.

## What to Extract

| Source | Extract As | PageType | Template? |
|--------|------------|----------|-----------|
| Code pattern | Pattern page | pattern | ✅ Yes |
| API pattern | Integration guide | howto | ✅ Yes |
| Decision (good/bad) | Decision page | decision | ❌ No |
| Failure / debugging | Concept page | concept | ❌ No |
| Error solution | Concept page | concept | ❌ No |
| Security approach | Concept page | concept | ❌ No |

## Checklist

- [ ] Source material reviewed
- [ ] Three categories analyzed (patterns, decisions, failures)
- [ ] Checked for existing wiki pages and memory to avoid duplicates
- [ ] Typed page created with correct page type (pattern/decision/concept) and `wm_page.create`
- [ ] Used appropriate template for the extraction type
- [ ] `references` edge linked back to source task via `wm_page.link`
- [ ] Quick memory entry created (summary + link to page)
- [ ] Promoted to critical-patterns if high-value (≥30 min save)
- [ ] Template created (if code-generatable)
- [ ] Validated (no broken refs)
- [ ] Linked back to source task

## Red Flags

- Only extracting code patterns, ignoring decisions and failures
- Saving incomplete or vague knowledge — future agents can't use it
- Duplicating existing knowledge — always search first
- Saving implementation details that will quickly become stale
- Not tagging pages — they won't surface in search
- Creating pages in wrong wiki subdirectory (use the type mapping)
- **Using `wm_doc.create` instead of `wm_page.create`** — typed pages enable graph traversal
- **Forgetting to add a `references` edge** — without it, the graph can't trace back to the source
- Saving personal preferences as project-wide patterns
- Promoting everything as critical (noise kills the learning loop)
- Fabricating findings when the task was straightforward

## Final Response Contract

All built-in skills in scope must end with the same user-facing information order: `wm-init`, `wm-spec`, `wm-plan`, `wm-research`, `wm-implement`, `wm-verify`, `wm-doc`, `wm-template`, `wm-extract`, and `wm-commit`.

Required order for the final user-facing response:

1. Goal/result - state what was accomplished.
2. Key details - include the most important supporting context, refs, assumptions, or validation.
3. Next action - recommend a concrete follow-up command only when a natural handoff exists.

Keep this concise for CLI use. Skill-specific content may extend the key-details section, but must not replace or reorder the shared structure.

Out of scope: explaining, syncing, or generating `.claude/skills/*`. Runtime auto-sync already handles platform copies, so this skill source only defines the built-in output contract.

For `wm-extract`, the key details should cover:
- what was extracted (pattern/decision/failure), where it was stored, related task or spec

## Related Skills

- `/wm-plan <task-id>` — Continue with next task
- `/wm-commit` — Commit extracted docs
- `/wm-flow` — Continue pipeline


## Next Step Suggestion

```
/wm-plan <task-id>     — Continue with next task
/wm-commit             — Commit extracted docs
/wm-go                 — Continue pipeline
```
