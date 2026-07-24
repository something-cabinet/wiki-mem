---
title: WM SDD Skills
type: spec
tags:
  - spec
  - approved
  - skills
  - workflow
---
id: wiki:specs:wm-sdd-skills

## Overview

Replace the current 4 flat gh-* skills (gh-ingest, gh-plan, gh-implement, gh-commit) with 13 subdirectory-format wm-* skills following the Knowns SDD workflow. Skills adopt Knowns' patterns: subdirectory format (`wm-*/SKILL.md`), `rust-embed` for binary embedding, Knowns' sync model (init doesn't touch skills, `wm setup` syncs), and full platform skill-dir mapping.

## Locked Decisions

- **D1**: Use `rust-embed` crate to embed skill files as a directory tree at compile time (Go `//go:embed` equivalent)
- **D2**: Full rewrite of all 13 skill files with WM-native `wm_*` tool references (not adapted from Knowns content)
- **D3**: Knowns sync model — `wm init` skips skills, `wm setup <platform>` syncs them. `wm setup --all` or `wm sync --skills` re-syncs.
- **D4**: Full platform skill-dir mapping (per platform, not shared):
  - `.claude/skills/` → claude-code
  - `.opencode/skills/` → opencode
  - `.codex/skills/` → codex
  - `.agents/skills/` → antigravity
  - `.agent/skills/` → agents (generic fallback)
  - `.kiro/skills/` → kiro
  - `.cursor/skills/` → cursor
  - (gemini uses platform-managed config)

## Requirements

### Functional Requirements

- **FR-1**: Skill parser in `skill.rs` handles `wm-*/SKILL.md` subdirectory format — uses parent directory name as skill name, not `file_stem("SKILL.md")`
- **FR-2**: Skill parser reads `name:` frontmatter field as primary name source (Knowns convention)
- **FR-3**: 13 wm-* skill files embedded via `rust-embed` derive macro, stored in `wm-core/src/skills/` directory
- **FR-4**: `wm setup <platform>` syncs embedded skills to the correct platform skill directory based on D4 mapping
- **FR-5**: `wm setup all` syncs skills to all target directories
- **FR-6**: `wm init` does NOT generate or sync skills (matches Knowns model)
- **FR-7**: `wm sync --skills` re-syncs embedded skills (future: `wm sync` command)
- **FR-8**: `sync_skills_to()` handles subdirectory structure recursively (not flat file copy)
- **FR-9**: Remove `generate_default_skills()` and all gh-* references
- **FR-10**: `wm serve` scans `.agents/skills/` for runtime skill loading (unchanged behavior)
- **FR-11**: Skills registered as MCP tools with `wm_skill.<name>` prefix (unchanged format)
- **FR-12**: Backward compat: existing skills in `.agents/skills/` are not deleted on upgrade

### Non-Functional Requirements

- **NFR-1**: `cargo build` and `cargo test` pass
- **NFR-2**: Existing unit tests in `skill.rs` updated to match new format
- **NFR-3**: New tests for subdirectory parsing, `name:` field, `rust-embed` loading, platform sync

## Acceptance Criteria

- [ ] AC-1: `wm-*/SKILL.md` subdirectory format parsed correctly — parent dir name becomes skill name
- [ ] AC-2: `name:` frontmatter field used when present (with fallback to parent dir name)
- [ ] AC-3: 13 wm-* skills embedded via `rust-embed` and loadable at runtime
- [ ] AC-4: `wm setup claude` syncs skills to `.claude/skills/` as subdirectories
- [ ] AC-5: `wm setup opencode` syncs skills to `.opencode/skills/` as subdirectories
- [ ] AC-6: `wm setup kiro` syncs skills to `.kiro/skills/` as subdirectories
- [ ] AC-7: `wm setup codex` syncs to `.codex/skills/`, `wm setup antigravity` syncs to `.agents/skills/`, `wm setup agents` syncs to `.agent/skills/`
- [ ] AC-8: `wm setup all` syncs to all platform target dirs
- [ ] AC-9: `wm init` does not generate any gh-* skills
- [ ] AC-10: `generate_default_skills()` removed, gh-* strings eliminated
- [ ] AC-11: `sync_skills_to()` copies subdirectory trees recursively
- [ ] AC-12: `wm serve` loads skills from `.agents/skills/` at startup
- [ ] AC-13: Existing skills in `.agents/skills/` survive upgrade (no cleanup)
- [ ] AC-14: `cargo build` + `cargo test` pass
- [ ] AC-15: Unit tests for subdirectory parse, `name:` field, platform sync

## Scenarios

### Scenario 1: New project init + setup

**Given** a fresh project directory
**When** user runs `wm init` followed by `wm setup opencode`
**Then** `.opencode/skills/wm-init/SKILL.md` through `.opencode/skills/wm-template/SKILL.md` exist (13 skills)
**And** `wm serve` lists 13 `wm_skill.*` MCP tools

### Scenario 2: Skill loaded as MCP tool

**Given** a platform skill dir (e.g. `.opencode/skills/wm-plan/SKILL.md`) exists with `name: wm-plan`
**When** `wm serve` starts
**Then** `tools/list` returns `wm_skill.wm-plan` with correct description
**And** calling `wm_skill.wm-plan` returns the skill instructions

### Scenario 3: Platform-specific sync

**Given** a WM project
**When** user runs `wm setup claude`
**Then** `.claude/skills/wm-init/SKILL.md` (etc.) exist as copies
**And** `sync_skills_to()` preserves subdirectory structure

### Scenario 4: Backward compat

**Given** a project with existing custom skills in `.agents/skills/`
**When** user runs `wm setup opencode`
**Then** custom skills are NOT deleted
**And** only the embedded wm-* skills are written/overwritten

## Open Questions

- [ ] Should `wm setup` prompt before overwriting existing skills? (Knowns uses `--force` flag)
- [ ] What replaces `wm init` current behavior of generating default skills — just remove that step silently?
- [ ] Should there be a `wm sync` command, or is `wm setup all` sufficient?

## Technical Notes

### Skill directory structure

```
wm-core/src/skills/
  wm-init/SKILL.md
  wm-research/SKILL.md
  wm-plan/SKILL.md
  wm-spec/SKILL.md
  wm-implement/SKILL.md
  wm-review/SKILL.md
  wm-commit/SKILL.md
  wm-verify/SKILL.md
  wm-doc/SKILL.md
  wm-extract/SKILL.md
  wm-debug/SKILL.md
  wm-go/SKILL.md
  wm-template/SKILL.md
```

### Parser changes (skill.rs)

Current `parse_skill_file()` uses `file_stem()` which gives `"SKILL"` for subdirectory files. Fix: detect parent directory for subdirectory format, add `name:` frontmatter field read.

### Sync changes (main.rs)

Current `sync_skills_to()` does flat `std::fs::copy` of files only. New version walks embedded skills tree, creates subdirectory structure at target, copies each file.

### Platform mapping

Each platform has its own native skill directory (not a shared one):
```rust
fn platform_skill_dir(platform: &str) -> &str {
    match platform {
        "claude-code" | "claude" => ".claude/skills",
        "opencode" => ".opencode/skills",
        "codex" => ".codex/skills",
        "kiro" => ".kiro/skills",
        "antigravity" => ".agents/skills",
        "cursor" => ".cursor/skills",
        "agents" => ".agent/skills",
        _ => ".agent/skills", // generic fallback
    }
}
```
