---
title: Embed shim templates via RustEmbed
id: a9a1fb
type: task
status: done
acceptance_criteria:
  - text: "apps/wm-core/src/shim_templates/ contains template files for AGENTS.md, CLAUDE.md, GEMINI.md, OPENCODE.md, REASONIX.md and .github/copilot-instructions.md with a RustEmbed struct"
  - text: "sync_agent_files() reads from embedded assets instead of format!() strings, and hardcoded template content is removed from main.rs"
  - text: "wm setup generates correct shim files for all platforms"
---

schema_version: 1
state: |-
  Shim files (AGENTS.md, CLAUDE.md, GEMINI.md, OPENCODE.md, REASONIX.md, .github/copilot-instructions.md) are currently constructed in `sync_agent_files()` using hardcoded `format!()` strings in `apps/wm-cli/src/main.rs`. They should instead be embedded at compile time via `rust_embed::RustEmbed` and copied out, matching the existing pattern used for skills.

  ## Acceptance Criteria

  - [x] Create `apps/wm-core/src/shim_templates/` with template files for each shim
  - [x] Add `RustEmbed` struct for shim templates
  - [x] Update `sync_agent_files()` to read from embedded assets instead of format strings
  - [x] Remove hardcoded template content from `main.rs`
  - [x] Verify `wm setup` generates correct shim files for all platforms

  ## References

  - Skills embedding: `apps/wm-core/src/skill/constants/skill_assets_constant.rs`
  - Current generator: `apps/wm-cli/src/main.rs:514-561`
questions:
  - id: work_kind
    type: choice
    instructions: What kind of work is this task?
    options:
    - feature
    - bugfix
    - refactor
    - docs
    - test
    - chore
    - migration
  - id: priority
    type: choice
    instructions: What priority is this task?
    options:
    - low
    - medium
    - high
    - urgent
  - id: needs_spec
    type: noul
    instructions: This task depends on a spec.
  - id: has_ac
    type: noul
    instructions: This task has at least one acceptance criterion.
answers: {}
