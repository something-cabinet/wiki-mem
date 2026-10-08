---
title: SA-02 Migrate all remaining page types
type: task
id: "wiki:tasks:sa-02-migrate-all-remaining-page-types"
status: todo
priority: high
tags: [from-spec, spec:structured-all-doc-types, migration, destructive]
spec: specs/structured-all-doc-types
acceptance_criteria:
  - text: "Migration converts all remaining page types (except index.md)"
  - text: "Per-type counts reported; 0 parse failures; frontmatter byte-preserved"
  - text: "Re-run is a no-op"
  - text: "Rule/memory/task prose preserved in state"
---

Extend the migration to all page types and convert the remaining pages (rule/core/memory/task/spec/note). Depends on SA-01. Per specs/structured-all-doc-types.