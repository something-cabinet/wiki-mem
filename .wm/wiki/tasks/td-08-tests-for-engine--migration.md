---
title: TD-08 Tests for engine + migration
type: task
id: "wiki:tasks:td-08-tests-for-engine--migration"
status: done
priority: high
tags: [from-spec, spec:typed-decision-doc-format, testing]
spec: specs/typed-decision-doc-format
acceptance_criteria:
  - text: "Tests for parser, validation, round-trip, migration idempotency"
  - text: "No regression in page read/graph/search"
relates_to:
  - {type: relates_to, target: wiki:specs:typed-decision-doc-format}
---

Tests covering TD-03..TD-06 and AC-3/4/8/9. Depends on engine + migration.