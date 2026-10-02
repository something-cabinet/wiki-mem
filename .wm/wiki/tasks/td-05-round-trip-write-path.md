---
title: TD-05 Round-trip write path
type: task
id: "wiki:tasks:td-05-round-trip-write-path"
status: done
priority: high
tags: [from-spec, spec:typed-decision-doc-format, engine]
spec: specs/typed-decision-doc-format
acceptance_criteria:
  - text: "parse -> write -> parse yields identical record + frontmatter"
  - text: "Uses line-based helpers; never whole-block YAML round-trip"
  - text: "id stays double-quoted"
relates_to:
  - {type: relates_to, target: wiki:specs:typed-decision-doc-format}
---

Implement the record write path preserving frontmatter, using line-based helpers. Depends on TD-01/TD-03.