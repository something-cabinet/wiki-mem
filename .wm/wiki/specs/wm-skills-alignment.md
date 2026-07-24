---
id: wiki:specs:wm-skills-alignment
title: WM Skills Alignment — Match KN Format
type: spec
status: approved
tags:
  - spec
  - skills
  - alignment
---
id: wiki:specs:wm-skills-alignment

# Spec: WM Skills Alignment — Match KN Format

## Overview

The 15 embedded skill files under `wm-core/src/skills/` (`wm-*`) are structurally inconsistent with their `kn-*` counterparts in the Knowns format. The previous session analyzed all 13 matching pairs and found that WM skills are missing several standard sections that KN skills have. This spec covers adding those missing sections to bring WM skills to parity.

## Locked Decisions

- **D-1:** Use the `kn-*` skills in `~/.config/opencode/skills/` as the reference format for structure
- **D-2:** Add `FinalResponseContract` section to skills that match their KN counterpart having it
- **D-3:** Add `RelatedSkills` section to skills that match their KN counterpart having it
- **D-4:** Add `Checklist` section to all WM skills (KN has it on most)
- **D-5:** Keep existing WM improvements (NextStep, RedFlags where WM already exceeds KN)
- **D-6:** Update the embedded source at `wm-core/src/skills/`, then run `wm setup opencode` to sync to platform dirs

## Requirements

### Functional Requirements

- FR-1: Add `FinalResponseContract` section to wm-plan, wm-implement, wm-init, wm-commit (matching kn-plan, kn-implement, kn-init, kn-commit)
- FR-2: Add `RelatedSkills` section to wm-plan, wm-implement, wm-review, wm-debug (matching kn-plan, kn-implement, kn-review, kn-debug)
- FR-3: Add `Checklist` section to all 15 WM skills
- FR-4: Keep all existing sections that WM already has (NextStep, RedFlags, Core principle, etc.)
- FR-5: Update tool references from `mcp_knowns_*` to `wm_*` in all sections
- FR-6: Run `wm setup opencode` after editing to sync skills to platform directories
- FR-7: wm-flow and wm-validate have no KN counterparts — ensure they match the general KN format pattern but keep their unique content

### Non-Functional Requirements

- NFR-1: All 15 skills must pass `wm lint check` after edits
- NFR-2: Each skill's `name` field in frontmatter must match its filename (e.g., `wm-spec` for `wm-spec/SKILL.md`)
- NFR-3: No regressions in existing skill behavior

## Acceptance Criteria

- [ ] AC-1: 4 WM skills have `FinalResponseContract` section matching KN format
- [ ] AC-2: 4 WM skills have `RelatedSkills` section matching KN format
- [ ] AC-3: All 15 WM skills have `Checklist` section
- [ ] AC-4: All existing WM sections (NextStep, RedFlags, Core principle) preserved
- [ ] AC-5: Tool references use `wm_*` not `mcp_knowns_*`
- [ ] AC-6: `wm setup opencode` runs without error and syncs to `.agent/skills/`
- [ ] AC-7: `cargo build -p wm-core` compiles cleanly

## Scenarios

### Happy Path
**Given** all 15 WM skill files in `wm-core/src/skills/`
**When** the missing sections are added and `wm setup opencode` runs
**Then** `.agent/skills/` and `.claude/skills/` contain the aligned skills
**And** all agents loading these skills see the complete structured format

### Edge Case: No KN counterpart
**Given** `wm-flow` and `wm-validate` have no `kn-*` match
**When** adding sections to these skills
**Then** use the general KN format pattern (Checklist, RedFlags) without requiring a specific template

## Delivery Notes

Skills to modify (source: `wm-core/src/skills/*/SKILL.md`):

| Skill | Add FinalResponse | Add RelatedSkills | Add Checklist |
|-------|------------------|------------------|--------------|
| wm-spec | ✅ (already done in .claude copy, needs source sync) | ✅ (already done) | ✅ |
| wm-plan | ✅ | ✅ | — |
| wm-implement | ✅ | ✅ | ✅ |
| wm-init | ✅ | — | — |
| wm-commit | ✅ | — | ✅ |
| wm-review | — | ✅ | ✅ |
| wm-debug | — | ✅ | ✅ |
| wm-research | — | — | ✅ |
| wm-doc | — | — | ✅ |
| wm-extract | — | — | ✅ |
| wm-go | — | — | ✅ |
| wm-template | — | — | ✅ |
| wm-verify | — | — | ✅ |
| wm-flow | — | — | ✅ |
| wm-validate | — | — | ✅ |

The `FinalResponseContract` template to add:

```markdown
## Final Response Contract

Required order for the final user-facing response:

1. **Goal/result** — state what was accomplished
2. **Key details** — include the most important supporting context, refs, open questions, or validation
3. **Next action** — recommend a concrete follow-up command only when a natural handoff exists
```

The `RelatedSkills` section to add where applicable:

```markdown
## Related Skills

- `/wm-plan ...` — Plan implementation
- `/wm-implement ...` — Implement changes
- etc. (relevant to skill domain)
```

## Open Questions

- [ ] Should `wm-flow` and `wm-validate` get their own `RelatedSkills` sections pointing to each other and to related skills?
