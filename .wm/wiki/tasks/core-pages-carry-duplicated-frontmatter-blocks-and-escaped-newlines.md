---
title: Core pages carry duplicated frontmatter blocks and escaped newlines
type: task
id: "wiki:tasks:core-pages-carry-duplicated-frontmatter-blocks-and-escaped-newlines"
status: todo
priority: medium
tags: [bug, wiki-health, frontmatter, core-pages]
acceptance_criteria:
  - text: "wiki:core:ARCHITECTURE body contains exactly one frontmatter block (no duplicate YAML block echoed into the body)"
  - text: "wiki:core:CONVENTIONS body contains exactly one frontmatter block (currently three)"
  - text: "wiki:core:critical-patterns stores real newlines instead of literal backslash-n escape sequences"
  - text: "A lint or validate rule detects duplicated frontmatter blocks and literal escape-sequence bodies so the class of defect cannot recur"
  - text: "Affected pages are repaired through wm_page update (never manual edits) and the index is rebuilt"
implementation_notes: |-
  2026-08-17 — full list from `wm_validate.check({})`, which fails with exactly these 7 errors. The task originally named 3 pages from spot observation; validation shows more, and one is a rule-adjacent page:

  - wiki:core:ARCHITECTURE
  - wiki:core:CONVENTIONS
  - wiki:patterns:line-based-frontmatter-editing
  - wiki:specs:linus-core-simplicity-rule
  - wiki:specs:retire-wm-doc
  - wiki:decisions:clippy-lint-curated-list-not-all
  - wiki:concepts:memory-system

  Note the irony worth preserving: `wiki:patterns:line-based-frontmatter-editing` is the page that documents how to avoid frontmatter corruption, and it is itself corrupted with duplicate blocks. `wiki:specs:linus-core-simplicity-rule` is also listed in wiki:tasks:fix-pre-existing-wiki-frontmatter-parse-errors for a different defect, so the two tasks overlap on that page.

  These 7 make `wm_validate.check({})` fail repo-wide, which means the wm-commit skill's abort condition (do not commit with validation errors) is unsatisfiable until they are fixed — every commit either violates the rule or ignores it. That raises the priority: it is not cosmetic, it blocks a documented gate.

  Pages created in the 2026-08-14/17 session were verified individually and validate clean, so the failure set is not growing.
---

Observed during wm-init core page reads (2026-08-14): wm_page get on wiki:core:ARCHITECTURE returns the frontmatter block twice (once as frontmatter, once echoed at the top of the body); wiki:core:CONVENTIONS returns it three times; wiki:core:critical-patterns stores its whole body with literal backslash-n sequences instead of real newlines, so the page renders as one unbroken line. This inflates every core-doc read for agents (wm-init reads all core pages every session) and corrupts critical-patterns readability. Related prior art: pattern line-based-frontmatter-editing and the duplicate-block validator rules mentioned there — either a writer regressed or these pages were written before the validator existed.