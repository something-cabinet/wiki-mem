---
title: Create platform_service.rs with template loading and merge logic
id: c2b0aa
type: task
status: done
priority: high
tags:
- from-spec
- spec:platform-embed-files
acceptance_criteria:
- text: platform_service.rs exists in wm-core/src/
- text: write_merged_json() function moved from main.rs
- text: write_toml_config() function moved from main.rs
- text: Functions are public and usable from wm-cli
- text: cargo build compiles
---

schema_version: 1
state: |-
  Create apps/wm-core/src/platform_service.rs module that provides template loading from EmbeddedFiles, JSON merging (write_merged_json), and TOML config writing (write_toml_config). Move the existing write_merged_json() and write_toml_config() functions from wm-cli/src/main.rs into this module.
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
