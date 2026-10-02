---
title: TD-06 One-shot migration of target docs
type: task
id: "wiki:tasks:td-06-one-shot-migration-of-target-docs"
status: done
priority: high
tags: [from-spec, spec:typed-decision-doc-format, migration, destructive]
spec: specs/typed-decision-doc-format
acceptance_criteria:
  - text: "All target pages converted; per-type count reported; 0 parse failures"
  - text: "Frontmatter preserved byte-for-byte"
  - text: "Re-run is a no-op (idempotent)"
  - text: "Respects the TD-02 exclusion list"
relates_to:
  - {type: relates_to, target: wiki:specs:typed-decision-doc-format}
---

Build and run the one-shot migration converting target docs to typed-decision records. Depends on TD-01/02/03/05. Destructive — gated on exclusion list.