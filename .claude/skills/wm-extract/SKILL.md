---
name: wm-extract
description: Extract reusable patterns, decisions, and failures into wiki pages
---

# Extracting Knowledge (with Compounding)

**Announce:** "Using wm-extract for [pattern/decision]."

**Core principle:** IF IT COST TIME TO LEARN, SAVE IT FOR LATER.

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
- Promote critical learnings to `learnings/critical-patterns` for future `wm-init` sessions

## Wiki Page Type Mapping

| Extraction Type | Wiki Subdirectory | PageType |
|----------------|-------------------|----------|
| Pattern | `patterns/` | Pattern |
| Decision | `decisions/` | Decision |
| Convention | `patterns/` | Pattern |
| Failure / Learning | `learnings/` | Learning |
| How-to | `howto/` | Howto |
| Reference | `reference/` | Reference |

Wiki pages are stored in the WM wiki and accessible via `wm_page.get({"path": "<subdir>/<slug>"})`.

## Step 1: Review Source Material

```json
wm_tasks.get({"taskId": "$ARGUMENTS"})
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
wm_search.search({"query": "<topic>", "type": "all", "mode": "keyword"})
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

### For patterns → Pattern Doc

```json
wm_docs.create({"title": "Pattern: <Name>",
  "folder": "patterns",
  "tags": ["pattern", "<domain>"],
  "content": "<markdown content>"})
```

#### Pattern Template

```markdown
## Problem
What problem does this pattern solve?

## Solution
The reusable approach or implementation.

## When to Use
Signals that indicate this pattern applies.

## When Not to Use
Contexts where this pattern adds unnecessary complexity.

## Related
- @page/patterns/...
- @task-...
```

### For decisions + failures → Learning Doc

```json
wm_docs.create({"title": "Learning: <feature/domain>",
  "folder": "learnings",
  "tags": ["learning", "<domain>"],
  "content": "<see template below>"})
```

#### Learning Doc Template

```markdown
## Patterns

### <Pattern Name>
- **What:** <description>
- **When to use:** <applicable conditions>
- **Source:** @task-<id>

## Decisions

### <Decision>
- **Chose:** <what was chosen>
- **Over:** <what was rejected>
- **Tag:** GOOD_CALL / BAD_CALL / SURPRISE / TRADEOFF
- **Outcome:** <how it played out>
- **Recommendation:** <for future work>

## Failures

### <Failure>
- **What went wrong:** <description>
- **Root cause:** <not just symptom>
- **Time lost:** <estimate>
- **Prevention:** <what to do differently>
```

### Decision Template (standalone)

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
- @page/decisions/...
- @task-...
```

## Step 5: Save Quick Memory (recall aid)

For each extracted pattern or decision worth quick recall, save a concise memory entry alongside the doc:

```json
wm_memory.add({"title": "<pattern/decision name>",
  "content": "<2-3 sentence summary>. Full reference: @doc/<path>",
  "layer": "project",
  "category": "<pattern|decision|convention|failure>",
  "tags": ["<domain>"]})
```

Memory = fast agent recall in future sessions. Doc = full structured reference.
Do NOT duplicate the entire doc content — store a summary and link to the doc.
Skip this step if the extraction produced nothing generalizable.

## Step 6: Create Template (if code-generatable)

```json
wm_templates.create({"name": "<pattern-name>",
  "description": "Generate <what>",
  "doc": "patterns/<pattern-name>"})
```

## Step 7: Promote to Critical

If the knowledge meets ALL criteria:
- Affects more than one future feature
- Would cause **≥30 minutes** wasted effort if unknown
- Is generalizable, not implementation-specific

Check if critical-patterns doc exists:

```json
wm_search.search({"query": "critical patterns", "type": "doc", "tag": "learning"})
```

**If exists — append:**

```json
wm_docs.update({"path": "learnings/critical-patterns",
  "appendContent": "\n\n## [Date] <Learning Title>\n**Category:** pattern / decision / failure\n**Source:** @task-<id>\n**Tags:** [tag1, tag2]\n\n<2-4 sentence summary and what to do differently>\n\n**Full entry:** @doc/learnings/<slug>"})
```

**If not exists — create:**

```json
wm_docs.create({"title": "Critical Patterns",
  "description": "Promoted learnings that save the most time. Read at session start.",
  "folder": "learnings",
  "tags": ["learning", "critical"],
  "content": "# Critical Patterns\n\nPromoted learnings from completed work. Read this at the start of every session via `wm-init`. These are lessons that cost the most to learn and save the most by knowing.\n\n---"})
```

**Calibration:** Do NOT promote everything. If critical-patterns grows past 20-30 entries it becomes noise. Only promote learnings that would have saved ≥30 minutes if known in advance.

## Step 8: Validate

```json
wm_validate({ "entity": "<doc-path>" })
```

If errors found, fix before continuing.

## Step 9: Link Back to Task

```json
wm_tasks.update({"taskId": "$ARGUMENTS",
  "appendNotes": "Extracted to @doc/<path>"})
```

---

# Consolidation Mode (Dream Lite)

When `$ARGUMENTS` contains `--consolidate`:

**Announce:** "Using wm-extract --consolidate to review and consolidate learnings."

Scan all existing learnings docs, merge duplicates, flag outdated entries, and promote new critical patterns. Run on-demand when the learnings folder feels messy or after a batch of completed work.

## C-Step 1: Scan All Learnings

```json
wm_docs.list({"tag": "learning"})
```

Read each learning doc:

```json
wm_docs.get({"path": "<path>", "smart": true})
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

| Source | Extract As | Template? |
|--------|------------|-----------|
| Code pattern | Pattern doc | ✅ Yes |
| API pattern | Integration guide | ✅ Yes |
| Decision (good/bad) | Learning doc | ❌ No |
| Failure / debugging | Learning doc | ❌ No |
| Error solution | Troubleshooting | ❌ No |
| Security approach | Guidelines | ❌ No |

## Checklist

- [ ] Source material reviewed
- [ ] Three categories analyzed (patterns, decisions, failures)
- [ ] Checked for existing wiki pages and memory to avoid duplicates
- [ ] Wiki page created in correct subdirectory (`patterns/`, `learnings/`, `concepts/`, etc.)
- [ ] Used appropriate template for the extraction type
- [ ] Quick memory entry created (summary + link to doc)
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
- Saving personal preferences as project-wide patterns
- Promoting everything as critical (noise kills the learning loop)
- Fabricating findings when the task was straightforward

## Next Step Suggestion

```
/wm-plan <task-id>     — Continue with next task
/wm-commit             — Commit extracted docs
/wm-go                 — Continue pipeline
```
