---
title: Platform Embed Files Restructure
type: spec
tags:
- spec
- platform
- embed
- refactor
- knowns
status: approved
implementation_notes: '## Related Tasks - @wiki/tasks/d41ec7 — Remove wm_template.run MCP tool and supporting template_engine module (separate cleanup discovered during spec research)'
---

schema_version: 1
state: |-
  id: wiki:specs:platform-embed-files

  ## Technical Notes

  - New module: `apps/wm-core/src/embed_files.rs` — single RustEmbed struct
  - New module: `apps/wm-core/src/platform_service.rs` — template loading, merge logic, config writing
  - Config templates are static (no placeholder substitution needed — `wm-cli` is always on PATH)
  - Move `write_merged_json()` and `write_toml_config()` from main.rs to `platform_service.rs`
  - Update imports in `skill_frontmatter_parser_helper.rs` and `wm-cli/src/main.rs`
  - No new dependencies required

  ## Open Questions

  None — all resolved.
questions:
  - id: kind
    type: choice
    instructions: What kind of spec is this?
    options:
    - feature
    - system
    - doc
    - migration
    - experiment
  - id: scope
    type: choice
    instructions: How wide is the scope of this spec?
    options:
    - local
    - component
    - system
    - project-wide
  - id: status_class
    type: choice
    instructions: What lifecycle class is this spec in?
    options:
    - draft
    - reviewed
    - approved
    - superseded
  - id: needs_tasks
    type: noul
    instructions: This spec requires one or more task pages.
answers: {}
