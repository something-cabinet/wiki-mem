---
title: IX-02 Index structured frontmatter + harden state extract
type: task
id: "wiki:tasks:ix-02-index-structured-frontmatter--harden-state-extract"
status: done
priority: high
tags: [from-spec, spec:reinforce-indexing-flow, search-quality, p0]
spec: specs/reinforce-indexing-flow
acceptance_criteria:
  - text: "Structured frontmatter (FR/NFR/goals, AC, decision context/options/rationale, aliases) indexed into sections/BM25"
  - text: "record_state_text tolerates leading blanks/extra keys; never indexes raw YAML"
  - text: "A requirement phrase from a spec is findable via keyword search"
---

Index structured frontmatter fields into sections/docs and harden record_state_text. Per specs/reinforce-indexing-flow FR-2.