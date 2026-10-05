---
title: TD-14 Fix gliner backend silent missing-task degradation
type: task
id: "wiki:tasks:td-14-fix-gliner-backend-silent-missing-task-degradation"
status: done
priority: high
tags: [from-review, spec:typed-decision-doc-format, p1, runtime]
spec: specs/typed-decision-doc-format
acceptance_criteria:
  - text: "A missing task/label result from the backend is an error, not a fabricated labels[0] p=0.0"
  - text: "DecisionRuntime's MissingTaskResult guard can fire for the real backend"
  - text: "Test covers the missing-task case"
---

Review P1-2: gliner backend degrades a missing task result to labels[0] p=0.0 (silent wrong answer). Return an error instead.