---
id: omuamh
title: wm init parity with knowns init
status: done
priority: medium
labels:
  - cli
  - init
  - knowns
  - parity
createdAt: '2026-07-06T18:23:34.424Z'
updatedAt: '2026-07-06T18:33:22.754Z'
timeSpent: 0
spec: specs/wm-init-platform-agent-instruction-files-mcp-config
---
# wm init parity with knowns init

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Make `wm init` behave identically to `knowns init`:

**Current `knowns init` behavior:**
- `knowns init` — interactive wizard that creates .knowns/ + AGENTS.md/CLAUDE.md + project config
- `knowns init <name>` — non-interactive, creates project with given name
- `knowns init <name> --no-wizard` — flag for explicit non-interactive mode
- `knowns init --platform <platform>` — generates platform-specific config (opencode.json, CLAUDE.md, etc.)
- Also syncs skills and agent instruction files

**Current `wm init` behavior (wm-cli/src/main.rs:403-510):**
- Creates `.wm/` directory + wiki subdirs (tasks, specs, concepts, patterns, decisions, howto, reference)
- Writes default config.json with ProjectConfig::default()
- Generates AGENTS.md (hardcoded string in main.rs)
- Generates default skill files via skill::generate_default_skills
- Has an unused `--platform` flag stub

**Gaps to fix:**
1. Accept project name as positional arg (not just --project path)
2. Add `--no-wizard` flag
3. Implement `--platform` flag to generate platform config (opencode.json entries, CLAUDE.md)
4. Generate a CLAUDE.md or CLI.md agent instruction file alongside AGENTS.md
5. Make the init process non-interactive by default when args given (match Knowns UX)
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 wm init <name> creates project with given name without prompting
- [x] #2 wm init <name> --no-wizard works the same (backward compat)
- [x] #3 wm init --platform <codex|claude> generates platform config (opencode.json or CLAUDE.md)
- [x] #4 CLAUDE.md is generated alongside AGENTS.md during init
- [x] #5 wm init without args still works (interactive or default behavior)
- [x] #6 cargo build + cargo test pass
- [x] #7 wm init --platform opencode generates OPENCODE.md + opencode.json with absolute binary path
- [x] #8 wm init --platform kiro generates KIRO.md + .kiro/settings/mcp.json with merge support
- [x] #9 wm init --platform claude|codex generates CLAUDE.md
- [x] #10 wm init --platform gemini generates GEMINI.md
- [x] #11 wm init --platform copilot generates .github/copilot-instructions.md
- [x] #12 wm init --platform agents confirms AGENTS.md exists
- [x] #13 Unknown platform name prints error with supported list
- [x] #14 All platform files reference .wm/AGENTS.md as canonical source of truth
- [x] #15 cargo build + cargo test pass
<!-- AC:END -->

