---
title: TD-03 Record parser + types
type: task
id: "wiki:tasks:td-03-record-parser--types"
status: done
priority: high
tags: [from-spec, spec:typed-decision-doc-format, engine]
spec: specs/typed-decision-doc-format
acceptance_criteria:
  - text: "Record types (state/questions/answers) parse from a page body"
  - text: "Parser tests cover choice/score/noul questions"
  - text: "No change to frontmatter parsing"
relates_to:
  - {type: relates_to, target: wiki:specs:typed-decision-doc-format}
---

Implement record parsing + types in the Rust engine: parse a page body into {state, questions[], answers}. Depends on TD-01.