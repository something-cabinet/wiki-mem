---
name: wm-review
description: Multi-perspective code review with wiki-backed context
---

# Review

**Announce:** "Using wm-review."

## Steps

### 1. Check wiki consistency
- `wm_lint.check()` — orphan pages, broken refs, stale sources
- `wm_validate.check()` — frontmatter completeness, graph connectivity

### 2. Check code quality
- Run `cargo build` / `cargo test`
- Review diff against existing wiki pages

### 3. Report findings
Group by severity: P1 (must fix), P2 (should fix), P3 (nice to fix)
