---
title: 'Pattern: Assert that a programmatic edit applied'
type: pattern
id: "wiki:patterns:assert-programmatic-edits-applied"
status: reviewed
tags: [pattern, tooling, editing]
relates_to:
  - {type: extends, target: wiki:patterns:verify-writes-by-read-back}
---

schema_version: 1
state: |-
  ## Problem

  A programmatic edit fails silently and the agent trusts it. A string or regex replacement that does not match returns the input unchanged; a partial parse yields partial output; a write reports success without landing. The edit *appears* to have happened and the agent proceeds as if it did.

  ## Solution

  Assert at both ends of every programmatic edit:

  1. **Before:** confirm the target text is present. `if (!body.includes(old)) abort;` A replacement whose anchor is missing is a no-op, not an error.
  2. **After:** confirm the new text is present. `if (!body.includes(new)) abort;` Do not write until this passes.
  3. Then **read the artifact back** from its store and re-check, because the write layer has its own failure modes - see @wiki/patterns:verify-writes-by-read-back.

  Anchors that produced misses in this session: assuming backticks or exact punctuation in the original that were not there; anchoring on a phrase that appears in several places; and building an anchor from what you *meant* to write rather than what was written.

  ## When to Use

  Any edit performed by code rather than by hand: find-and-replace over a document body, regex rewrites, bulk frontmatter edits, template substitution, migrations.

  ## When Not to Use

  Single hand-written edits where the change is visible in the diff - the assertions add noise without adding safety.

  ## Related

  - @wiki/patterns:verify-writes-by-read-back
  - @wiki/patterns:line-based-frontmatter-editing

  ## Addendum 2026-10-15 - assert your *extraction* too

  The same trap has a mirror: a helper that **extracts** a region can silently exclude the very text you meant to change. This session's body-extraction helper assumed every page carried a `questions:` block and sliced `lines[s+1 .. q)`. On a page *without* that block, `q` resolved to `-1`, so `slice(s+1, -1)` dropped the **last line** - which was the line holding the target reference. The replacement then matched nothing and reported "ref not in body": true, and misleading at the same time, because the ref was right there in the file.

  Guards:

  1. Assert the extracted region **contains the anchor** before you attempt the edit, and abort loudly if it does not - never fall through to a replacement that may silently no-op.
  2. When a lookup misses, suspect the **extractor's assumed shape** before concluding the text is absent. Pages here come in three shapes: full record (`state` + `questions`), partial (record body, no `questions`), and prose (no envelope at all).
  3. A negative slice index is the classic culprit: `slice(a, -1)` reads as "up to the end" but means "up to, excluding, the last line".

  Cost this time: four ref fixes reported as unfixable that were trivially fixable once the extractor stopped lying.
questions:
  - id: problem_kind
    type: choice
    instructions: What kind of problem does this pattern solve?
    options:
    - architecture
    - api-design
    - data-model
    - error-handling
    - performance
    - testing
    - ui
    - tooling
    - workflow
  - id: preconditions_required
    type: noul
    instructions: This pattern requires specific preconditions to be met.
  - id: complexity
    type: score
    instructions: How complex is applying this pattern?
    levels:
    - trivial
    - simple
    - moderate
    - complex
  - id: language_specific
    type: noul
    instructions: This pattern is specific to a programming language.
answers: {}