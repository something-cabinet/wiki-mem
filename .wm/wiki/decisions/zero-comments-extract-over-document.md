---
title: "Decision: Zero Comments — Extract Over Document"
type: decision
status: approved
tags: [decision, naming, quality, rule]
relates_to:
  - {type: references, target: wiki:rules:no-comments-in-code}
---

## Context

The project's no-comments rule originally prohibited inline comments explaining *what* code does (`// increment counter`) but carved out exceptions: doc comments (`///`, `/** */`), module docs (`//!`), section markers (`// ───`), and TODO/FIXME/HACK markers. A scan found ~1,000 comment lines across 90+ files — every category represented. The exceptions had eroded the rule.

## Decision

Zero comments. No exceptions. Every `//`, `///`, `//!`, `/** */`, `/* */`, `<!-- -->` line is a violation. TODO/FIXME/HACK markers are not exempt — file as WM tasks instead.

If a function, module, field, or block needs a comment to be understood, it should be split, extracted, or composed instead. Self-documenting names replace doc comments:

| Before | After |
|--------|-------|
| `/// Returns true if the terminal supports Unicode` | `fn terminal_supports_unicode() -> bool` |
| `/** Cache for parsed oklch [l, c, h] components */` | `oklchCache` (field name) |
| `// ─── Input types ───` (section marker) | Split into `input_types.rs` module |
| `// Skill system` (field label) | `skill_engine: SkillEngine` (the type already says it) |
| `// TODO: implement doc history compaction` | Filed as `.wm/wiki/tasks/implement-doc-history-compaction.md` |

## Rationale

- Comments rot, drift from code, and create false confidence — doc comments are not immune
- Zero-tolerance enforcement is simpler to check (grep for `//`) than a nuanced policy
- Named functions and compositional patterns are refactor-safe and always up to date
- TODOs in code are invisible to task tracking — WM tasks are discoverable via search and graph

## Consequences

- Code becomes self-documenting by necessity — better naming and composition
- ~1,000 comment lines removed from ~90 files in one pass
- API docs must move to wiki pages (pattern/decision/concept pages) — not `///` on functions
- Generated files (e.g., wasm-pack `.d.ts`) need explicit exclusion
- `// @ts-ignore` directives are kept as compiler directives (not code comments)

## Related
- @wiki/rules/no-comments-in-code (updated to reflect this decision)
- @wiki/tasks/strip-all-comments-from-source-code
