---
id: wiki:specs:reasonix-orchestrate
title: Reasonix Orchestrate
type: spec
status: approved
---
id: wiki:specs:reasonix-orchestrate

## Overview

A Rust CLI binary (`reasonix-orchestrate`) that installs an agent orchestrator system into any Reasonix project. It embeds 6 specialist skill files and an orchestrator prompt via `rust-embed`, then:

1. Copies skills to `.reasonix/skills/<name>/SKILL.md`
2. Writes `.reasonix/ORCHESTRATOR.md` with the full orchestrator prompt
3. Generates or amends `REASONIX.md` with a shim referencing the orchestrator
4. Updates global `%APPDATA%/reasonix/config.toml` with `subagent_models` and `[skills] paths`

No manual steps. One command, everything wired.

## Locked Decisions

- D1: Install + update + version management (not just a one-shot installer)
- D2: Overwrite without confirmation — always overwrite existing files
- D3: Single `init` command — no subcommands needed
- D4: `init` updates both project files AND global Reasonix config
- D5: REASONIX.md shim strategy — append ref if exists, create minimal shim if not. Full prompt in `.reasonix/ORCHESTRATOR.md`

## Requirements

### Functional Requirements

- FR-1: Binary embeds 6 skill SKILL.md files and 1 ORCHESTRATOR.md via `rust-embed`
- FR-2: `orchestrate init` copies all 6 skills to `.reasonix/skills/<name>/SKILL.md` in the current directory
- FR-3: `orchestrate init` writes `.reasonix/ORCHESTRATOR.md` with the orchestrator prompt
- FR-4: `orchestrate init` creates or amends `REASONIX.md`:
  - If `REASONIX.md` does not exist: create it with a shim pointing to `@.reasonix/ORCHESTRATOR.md`
  - If `REASONIX.md` exists: append a reference line at the bottom
- FR-5: `orchestrate init` updates global `%APPDATA%/reasonix/config.toml`:
  - Sets `[skills] paths` to include the Reasonix home skills dir
  - Sets `subagent_models` for all 6 specialist skills (explorer=flash, fixer=flash, librarian=flash, oracle=pro, designer=pro, reviewer=pro)
- FR-6: `—version` flag prints the current version (from `git describe —tags` via ldflags)
- FR-7: All file writes are unconditional overwrites — no diff, no confirmation prompts

### Non-Functional Requirements

- NFR-1: Single static binary, CGO-free, Windows x64 target
- NFR-2: No runtime dependencies beyond the OS
- NFR-3: `init` completes in under 1 second
- NFR-4: Embedded files are compiled into the binary, not read from disk at runtime

## Acceptance Criteria

- [ ] AC-1: `orchestrate init` in an empty directory creates `.reasonix/skills/` with 6 skill subdirectories, each containing `SKILL.md`
- [ ] AC-2: `.reasonix/ORCHESTRATOR.md` exists with the orchestrator prompt after `init`
- [ ] AC-3: `REASONIX.md` exists with shim text referencing `@.reasonix/ORCHESTRATOR.md`
- [ ] AC-4: Running `init` again overwrites all files without errors
- [ ] AC-5: `orchestrate —version` prints a semver string
- [ ] AC-6: Global `config.toml` has `subagent_models` and `[skills] paths` set after `init`
- [ ] AC-7: In a project that already has `REASONIX.md`, running `init` appends a line rather than overwriting the file

## Scenarios

### Scenario 1: Fresh project
**Given** a directory with no Reasonix config
**When** user runs `orchestrate init`
**Then** `.reasonix/skills/`, `REASONIX.md`, `.reasonix/ORCHESTRATOR.md` are created
**And** global `config.toml` is updated with subagent_models and skills path

### Scenario 2: Existing REASONIX.md
**Given** a project that already has a `REASONIX.md`
**When** user runs `orchestrate init`
**Then** `.reasonix/skills/` and `.reasonix/ORCHESTRATOR.md` are overwritten
**And** a reference line is appended to the existing `REASONIX.md`
**And** no prior content is lost

### Scenario 3: Re-run
**Given** a project already initialized
**When** user runs `orchestrate init` again (e.g., after updating the binary)
**Then** all files are overwritten with the embedded versions
**And** no errors occur

## Embedded Files

The binary embeds these files:

```
orchestrator/ORCHESTRATOR.md     — Full orchestrator prompt with 6 agent lanes
skills/explorer/SKILL.md        — Read-only code search specialist
skills/fixer/SKILL.md           — Bounded implementation specialist
skills/oracle/SKILL.md          — Architecture/review specialist
skills/librarian/SKILL.md       — Web research specialist
skills/designer/SKILL.md        — UI/UX design specialist
skills/reviewer/SKILL.md        — Code review specialist
```

## REASONIX.md Shim Content

When creating a new REASONIX.md:
```markdown
# Agent Orchestration

This project uses `reasonix-orchestrate` for agent lane management.
See `.reasonix/ORCHESTRATOR.md` for available specialists and delegation rules.
```

When appending to an existing REASONIX.md:
```markdown
> Agent orchestration managed by reasonix-orchestrate. See `.reasonix/ORCHESTRATOR.md` for specialist lanes.
```

## Global Config Changes

The binary adds to `%APPDATA%/reasonix/config.toml`:

```toml
[skills]
paths = ["C:\\Users\\hk\\AppData\\Roaming\\reasonix\\skills"]

[agent]
subagent_models = {
  explorer = "deepseek-v4-flash",
  fixer = "deepseek-v4-flash",
  librarian = "deepseek-v4-flash",
  oracle = "deepseek-v4-pro",
  designer = "deepseek-v4-pro",
  reviewer = "deepseek-v4-pro"
}
```

## Technical Notes

- Crate: `reasonix-orchestrate` in the workspace or standalone
- Dependencies: `rust-embed`, `clap` (for `—version`), `serde` + `toml` (for config.toml editing)
- Target: `x86_64-pc-windows-msvc` (matches existing WM toolchain)
- The global config edit should parse the existing TOML, merge in the new keys, and write back preserving comments where possible (or use `toml_edit` for non-destructive edits)

## Open Questions

- [ ] Should `init` also accept a `—dir` flag to target a different project directory?
- [ ] Should the binary also install the skills globally (to `%APPDATA%/reasonix/skills/`) or is that a separate step?