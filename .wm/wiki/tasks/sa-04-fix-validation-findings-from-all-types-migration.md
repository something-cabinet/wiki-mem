---
title: SA-04 Fix validation findings from all-types migration
type: task
id: "wiki:tasks:sa-04-fix-validation-findings-from-all-types-migration"
status: todo
priority: high
tags: [from-spec, spec:structured-all-doc-types, validation, wiki-health]
spec: specs/structured-all-doc-types
acceptance_criteria:
  - text: "13 excluded-dir prose pages resolved (convert or opt-out)"
  - text: "6 type/dir-mismatch pages corrected (frontmatter type vs directory)"
  - text: "3 malformed/duplicated-frontmatter pages repaired"
  - text: "2 parse-failure spec bodies repaired or opted out; 1 empty-state decision fixed"
  - text: "wm_validate.check all-scope returns 0 errors"
---

Fix the validation findings surfaced by the all-types migration: 13 prose pages in planner-excluded dirs (research/learnings/conventions), 6 pages whose frontmatter type differs from their directory (canonical mismatch), 3 malformed/duplicated frontmatter pages, 2 parse-failure specs (control chars / carriage return), 1 empty-state decision. `wm_validate.check` all-scope currently reports 25 errors; healthy converted pages validate clean.