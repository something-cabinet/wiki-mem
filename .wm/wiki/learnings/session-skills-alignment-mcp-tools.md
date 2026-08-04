---
title: 'Learning: MCP Tools Fix + Skills Alignment'
id: wiki:learnings:session-skills-alignment-mcp-tools
type: concept
relates_to:
  - {type: references, target: wiki:specs:wm-skills-alignment}
---

# Learning: MCP Tools Fix + Skills Alignment

## Patterns

### Parallel Fixer Agents for Batch File Editing
- **What:** When updating a consistent template across many files (e.g., adding the same section to 8+ skill files), spawn parallel `fixer` subagents each handling 2–8 files. Give each fixer the exact template text and per-file customizations. This completes the work in one roundtrip instead of sequential edits.
- **When to use:** Any batch operation where each file needs the same structural change with minor variations (adding sections, renaming references, updating templates)
- **Source:** @wiki/concepts/specs/wm-skills-alignment
- **Promoted to:** @wiki/patterns/parallel-fixer-agents (full pattern page)

### WM Embedded Skills Workflow
- **What:** Skills are embedded in the Rust binary via `rust-embed` at compile time. Source of truth is `apps/wm-core/src/skills/*/SKILL.md`. To update: edit source → `cargo build -p wm-cli` → `wm setup opencode` syncs to `.agent/skills/`.
- **When to use:** Any time a skill file needs to change
- **Source:** @wiki/concepts/specs/wm-skills-alignment

## Decisions

### Standardize FinalResponseContract + RelatedSkills on All WM Skills
- **Chose:** Add `FinalResponseContract` (structured output: Goal/result → Key details → Next action) and `RelatedSkills` section to all 15 WM skills
- **Over:** Leaving them in the old ad-hoc format
- **Tag:** GOOD_CALL
- **Outcome:** All WM skills now follow the same structured output format as the KN reference skills. Agents will produce consistent responses.
- **Recommendation:** When creating new skills, always include FinalResponseContract, RelatedSkills, Checklist, and RedFlags sections from the start.

### Repo Restructure: wm-core to apps/wm-core, wm-web to wm-server
- **Chose:** Moved `wm-core/src/` to `apps/wm-core/src/`, `wm-cli/` to `apps/wm-cli/`, renamed `wm-web` to `wm-server` as optional dependency
- **Tag:** SURPRISE (discovered mid-session, pre-existing change)
- **Outcome:** Cleaner monorepo layout, optional web feature fixes pre-existing build breakage

## Failures

### Spec Overestimated Checklist Gap
- **What went wrong:** The spec `wm-skills-alignment` estimated 8 skills needed Checklist adding, but a structural audit revealed all 15 already had it. The real gap was FinalResponseContract and RelatedSkills only.
- **Root cause:** Spec was written based on assumptions from a quick grep, not a thorough audit of actual file contents.
- **Time lost:** ~10 minutes fixing the delivery table
- **Prevention:** Always verify structural assumptions with a concrete file scan before writing the spec's delivery table.

## Related
- @wiki/concepts/specs/wm-skills-alignment
- @wiki/concepts/patterns/critical-patterns