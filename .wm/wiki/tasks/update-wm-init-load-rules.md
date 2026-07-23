---
title: "Update wm-init skill to load wiki rules at session start"
type: task
status: todo
severity: Medium
spec: wiki-rules-auto-load
fulfills: FR-1, FR-2, FR-3, FR-4, FR-5
relates_to:
  - {type: implements, target: wiki:specs:wiki-rules-auto-load}
---

## Task: Update wm-init skill

Add a "Step 4.5: Load Wiki Rules" to the wm-init skill that:
- Discovers all rule pages from `.wm/wiki/rules/` (MCP-first, file fallback)
- Reads each rule and summarizes in session context
- Adds a validation check against active rules before task completion

### Acceptance Criteria
- [ ] wm-init includes a dedicated step for loading wiki rules
- [ ] Uses MCP tools first, falls back to direct file reads
- [ ] Session context output includes a "Rules" section
- [ ] Validation step checks work against active rules
