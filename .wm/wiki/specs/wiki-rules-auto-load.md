---
title: "Wiki Rules Auto-Load at Session Start"
type: spec
status: approved
tags: ["spec", "workflow", "rules"]
---

## Overview

Make all wiki rules under `.wm/wiki/rules/` load automatically at session start and become binding guidance for the agent. Currently, rules are documented but agents do not discover them unless they explicitly search — this spec closes that gap by making rule-loading part of the boot sequence.

## Locked Decisions

- D1: Rules are loaded via both paths — wiki MCP tools first (`wm_page.list` + `wm_page.get` filtered to type=rule), direct file reads from `.wm/wiki/rules/` as fallback when MCP is unavailable.
- D2: Only pages under `.wm/wiki/rules/` count as rules. `decisions/` and `patterns/` are not loaded as rules.
- D3: Rules are loaded into session context AND a validation step checks work against active rules before marking complete.
- D4: All four compatibility shims (AGENTS.md, CLAUDE.md, GEMINI.md, OPENCODE.md) are updated to mention the rule-loading requirement.

## Requirements

### Functional Requirements

- FR-1: The `wm-init` skill must discover and read all rule pages at session start.
- FR-2: Rules must be loaded via MCP tools (`wm_page.list` with type filter, then `wm_page.get` for each).
- FR-3: When MCP tools are unavailable, fall back to reading `.wm/wiki/rules/*.md` directly.
- FR-4: Loaded rules must be summarized in the session context output so the agent (and user) see which rules are active.
- FR-5: A validation mechanism must check work against active rules before marking tasks complete.
- FR-6: All four compatibility shims must include a one-line instruction about rule loading for non-Reasonix agents.
- FR-7: WIKI-MEM.md must reference rule loading in both TL;DR and Critical Rules sections.

### Non-Functional Requirements

- NFR-1: Rule loading must not significantly increase session init time (target: <2s additional).
- NFR-2: A missing or empty rules/ directory must not crash init — gracefully report "no active rules."

## Acceptance Criteria

- [ ] AC-1: On session start, `wm-init` lists all pages in `.wm/wiki/rules/` and reads each one.
- [ ] AC-2: When MCP `wm_page` tools are available, they are used; when unavailable, direct file reads work.
- [ ] AC-3: Session context summary includes a "Rules" section listing each active rule and its key requirement.
- [ ] AC-4: Before task completion, the agent checks work against active rules and flags any violations.
- [ ] AC-5: AGENTS.md, CLAUDE.md, GEMINI.md, and OPENCODE.md each contain a rule-loading instruction.
- [ ] AC-6: WIKI-MEM.md TL;DR mentions rule loading; Critical Rules includes it.
- [ ] AC-7: When `rules/` is empty or missing, init completes without error and reports "no active rules."

## Scenarios

### Scenario 1: Happy Path — Rules Exist, MCP Available

**Given** the project has 3 rule files in `.wm/wiki/rules/`
**When** `wm-init` runs at session start
**Then** it lists all 3 rules via `wm_page.list`
**And** reads each one via `wm_page.get`
**And** the session context includes a "Rules" section summarizing all 3
**And** validation checks against rules before task completion

### Scenario 2: Fallback — MCP Unavailable

**Given** the wm-cli binary is not built or MCP tools are disconnected
**When** `wm-init` runs at session start
**Then** it falls back to reading `.wm/wiki/rules/*.md` files directly
**And** produces the same session context as Scenario 1

### Scenario 3: Empty Rules Directory

**Given** `.wm/wiki/rules/` is empty or missing
**When** `wm-init` runs
**Then** it completes without error
**And** reports "No active rules" in the session context

## Technical Notes

### Files to change

| File | Change |
|---|---|
| `.agent/skills/wm-init/SKILL.md` | Add "Step 4.5: Load Wiki Rules" between task board and critical learnings. Include MCP-first, files-fallback, validation check. Renumber subsequent steps. |
| `WIKI-MEM.md` | Add to TL;DR: "Load all wiki rules at session start and obey them." Add to Critical Rules: "Wiki rules under @wiki/rules/ are authoritative — load and obey every active rule." |
| `AGENTS.md` | Add one-line instruction: "Read all rules from .wm/wiki/rules/ at session start and obey them." |
| `CLAUDE.md` | Same one-line instruction. |
| `GEMINI.md` | Same one-line instruction. |
| `OPENCODE.md` | Same one-line instruction. |

### Rule format expectation

Rules are Markdown files with frontmatter:
```yaml
---
title: "..."
type: rule
status: active
---
```

Only rules with `status: active` must be obeyed. Inactive or draft rules are informational.

## Open Questions

- [ ] Should the validation step be integrated into an existing skill (e.g., wm-verify) or be a new standalone check?
