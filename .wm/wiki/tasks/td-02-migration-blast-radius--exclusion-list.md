---
title: TD-02 Migration blast-radius + exclusion list
type: task
id: "wiki:tasks:td-02-migration-blast-radius--exclusion-list"
status: done
priority: high
tags: [from-spec, spec:typed-decision-doc-format, risk, p0]
spec: specs/typed-decision-doc-format
acceptance_criteria:
  - text: "Identify every wiki page/section agents read as instructions or context"
  - text: "Recommend an explicit exclusion list (page types/paths) that must stay human-readable"
  - text: "Report with file:line refs; read-only, no writes"
relates_to:
  - {type: relates_to, target: wiki:specs:typed-decision-doc-format}
---

Blast-radius analysis for replacing page bodies with machine records: which docs the agent workflow consumes as binding instructions/context (rules, core pages, WIKI-MEM.md, skill embeds), and what must be excluded from conversion. Read-only.