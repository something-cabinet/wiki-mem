---
name: wm-review
description: Multi-perspective code review with severity-based findings
---

# Code Review

**Announce:** "Using wm-review."

**Core principle:** STRUCTURE → CLARITY → CORRECTNESS → CONSISTENCY.

## Review Perspectives

Review changed code through these lenses:

| Perspective | What to check |
|-------------|---------------|
| **Structure** | Architecture fit, module boundaries, dependency direction |
| **Correctness** | Edge cases, error handling, race conditions, type safety |
| **Clarity** | Naming, comments, function length, readability |
| **Consistency** | Project conventions, pattern reuse, style |

## Output

Group findings by severity:
- **P0** — Bug or incorrect behavior (must fix)
- **P1** — Design or clarity issue (should fix)
- **P2** — Style or minor concern (nice to fix)

Use wiki `wm_search.query` to check for existing patterns and conventions during review.
