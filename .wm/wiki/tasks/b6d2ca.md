---
title: 'P5c: Single-file section parsing'
id: b6d2ca
type: task
status: done
priority: medium
tags:
- from-spec
- spec:graph-connectivity-fix
- p5
acceptance_criteria:
- text: Single-file section parsing is extracted from build_sections_from_wiki into a standalone function
- text: The standalone section parser is wired into the incremental cascade
- text: FR-11 sections portion acceptance criteria from the graph-connectivity-fix spec are satisfied
---

Implement FR-11 sections portion. Extract single-file section parsing from build_sections_from_wiki into standalone function. Wire into incremental cascade.