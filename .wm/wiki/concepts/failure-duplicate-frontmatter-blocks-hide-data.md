---
title: 'Failure: Duplicate Frontmatter Blocks Hide Data from Parser'
type: concept
id: wiki:concepts:failure-duplicate-frontmatter-blocks-hide-data
status: draft
tags:
- failure
- frontmatter
- parser
- validation
- yaml
relates_to:
  - {type: references, target: wiki:tasks:fix-readme}
---

## What went wrong

During the wm_validate remediation (236 → 0 errors), one task (`wm-cli-web-false-started-when-stale-process-holds-the-port`) still failed validation with "Task should have at least one acceptance criterion" even though the file visibly contained `acceptance_criteria`. A parallel fixer had skipped it because it "already had ACs".

## Root cause

The file had TWO `---`-delimited YAML frontmatter blocks — the first block (title/id/type only, no ACs) followed by a second full frontmatter block in the body containing the real ACs. The wiki parser reads only the FIRST frontmatter block; content in later `---`-delimited blocks is body text and invisible to validation. The fixer inspected the file, saw ACs in the second block, and skipped it as already-complete.

Multiple task files carry this malformed duplicated-frontmatter structure (ef4616, wm-index-code-output-misleading, 2a335e, d93671, wm-cli-web-false-started, and others).

## Prevention

- When validating a task that "already has ACs" but validation disagrees: verify the ACs are in the FIRST frontmatter block, not a later one.
- When a fixer reports a skip with reason "already has X", spot-check the actual file — a skip with a confident reason can hide a parser-visibility bug.
- Clean-up candidates: task files with duplicate frontmatter blocks should be normalized to a single block (the first block is authoritative; merge or remove the second).

## Time lost

~15 minutes diagnosing one phantom validation error across a 236-error remediation.

## Related

- @wiki/tasks/fix-readme