---
title: Failure — Bulk frontmatter repair script destroyed wiki content
type: concept
id: wiki:concepts:failure-bulk-frontmatter-repair-data-loss
status: draft
---

## What went wrong

A repair script intended to fix 105 corrupted wiki pages DELETED 2,410 lines of content across 72 files — including 759 lines from `specs/onnx-embedding-integration.md` and 68 from `core/CONVENTIONS.md`.

## Root cause

The script treated every pair of `---` markers as a frontmatter block boundary. Many wiki pages use `---` as markdown section separators (horizontal rules). Content between those markers was swallowed into "frontmatter" and dropped.

The body-preservation guard was worthless because it only compared text AFTER the last paired marker — the region being corrupted was the content BETWEEN intermediate markers, which the script had already consumed.

## Prevention

1. Frontmatter is ONLY lines[0]==`---` to the FIRST subsequent `---`. Period. Every other `---` in the file is content.
2. Never write a bulk-edit script that touches tracked files without a BYTE-EXACT body assertion: `old_body == new_body` where body is everything after the first closing `---`.
3. Always run `git diff --numstat` immediately after and check for large negative deltas before proceeding.
4. The second script caught the mistake because it had this assertion and because `git diff` was checked within seconds.

## Time lost

~20 minutes (caught quickly because the diagnostic showed the damage immediately, and `git checkout` reverted all 105 files instantly).

## Related

- wiki:specs:security-remediation
- wiki:tasks:wm-task-update-frontmatter-corruption
