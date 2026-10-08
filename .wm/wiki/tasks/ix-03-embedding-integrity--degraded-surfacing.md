---
title: IX-03 Embedding integrity + degraded surfacing
type: task
id: "wiki:tasks:ix-03-embedding-integrity--degraded-surfacing"
status: done
priority: high
tags: [from-spec, spec:reinforce-indexing-flow, robustness]
spec: specs/reinforce-indexing-flow
acceptance_criteria:
  - text: "Persisted model/version validated on load; mismatch forces re-embed or clear"
  - text: "degraded + actionable reason surfaced in query/retrieve/index status"
---

Embedding integrity: validate persisted model/version on load, force re-embed/clear on mismatch, surface degraded reason everywhere. Per specs/reinforce-indexing-flow FR-3.