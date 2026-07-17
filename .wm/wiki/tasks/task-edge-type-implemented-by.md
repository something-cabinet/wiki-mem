---
title: Register custom edge type 'implemented-by' in config
type: task
status: todo
---

**Severity:** Low

**Observed:** Custom edge type `implemented-by` is not registered in `.wm/config.json` custom_edge_types. Edges of this type are skipped during graph rebuild.

**Root Cause:** The edge type `implemented-by` exists in wiki page frontmatter but isn't listed in `config.json`.

**Fix:** Add `"implemented-by"` to `custom_edge_types` array in `.wm/config.json`.

**Acceptance Criteria:**
- [ ] No "Custom edge type 'implemented-by' not registered in config" warning
- [ ] `implemented-by` edges appear in graph