---
title: system-one-03 Synthesize per-workflow verdicts + eval designs
type: task
id: "wiki:tasks:system-one-03-synthesize-per-workflow-verdicts--eval-designs"
status: done
priority: high
tags: [from-spec, spec:system-one-model-application, synthesis]
spec: specs/system-one-model-application
acceptance_criteria:
  - text: "FR-5: each workflow has an evaluation design with metric and pass threshold"
  - text: "FR-7: each workflow assigned GO / NO-GO / NEEDS-FINETUNE with rationale"
  - text: "FR-8: one first-prototype workflow recommended (or none stated)"
relates_to:
  - {type: relates_to, target: wiki:specs:system-one-model-application}
---

Synthesize Wave 1 findings into per-workflow verdicts and evaluation designs. Depends on system-one-01 and system-one-02.