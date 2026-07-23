---
title: "CLOSED — Register custom edge type 'implemented-by' in config"
type: task
status: cancelled
spec: specs/edge-type-pruning
superseded_by: wiki:specs:edge-type-pruning
relates_to:
  - {type: implements, target: wiki:specs:edge-type-pruning}
---

**Severity:** Low

**Resolution:** Won't-do per edge-type-pruning spec. Inverse-edge policy: canonical single direction + reverse traversal. The 2 `implemented-by` edges were rewritten as `implements` from the decision side (see decisions/init-setup-separation, decisions/error-response-format).

**Acceptance Criteria:**
- [ ] No "Custom edge type 'implemented-by' not registered in config" warning
- [ ] `implemented-by` edges appear in graph