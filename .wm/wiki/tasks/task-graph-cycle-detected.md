---
title: Investigate and resolve wiki graph cycle
type: task
status: todo
spec: specs/graph-bugs-review-fixes
---

**Severity:** Low

**Observed:** A cycle was detected in the wiki graph during rebuild. BFS uses visited tracking to prevent infinite loops.

**Impact:** Graph traversal (e.g. neighbors, path finding) may produce unexpected results around the cycle. The cycle itself isn't harmful but indicates a circular reference in `relates_to` frontmatter.

**Investigation needed:**
- Identify which pages form the cycle
- Determine if it's intentional (mutual references) or accidental
- Optionally break the cycle or document it

**Acceptance Criteria:**
- [ ] Graph cycle either resolved or documented as intentional
- [ ] No "Cycle detected in wiki graph" warning on startup (or explicit note if expected)