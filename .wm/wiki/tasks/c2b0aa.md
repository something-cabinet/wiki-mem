---
title: Create platform_service.rs with template loading and merge logic
id: c2b0aa
type: task
status: todo
priority: high
tags: [from-spec, spec:platform-embed-files]
acceptance_criteria:
  - text: "platform_service.rs exists in wm-core/src/"
  - text: "write_merged_json() function moved from main.rs"
  - text: "write_toml_config() function moved from main.rs"
  - text: "Functions are public and usable from wm-cli"
  - text: "cargo build compiles"
---

Create apps/wm-core/src/platform_service.rs module that provides template loading from EmbeddedFiles, JSON merging (write_merged_json), and TOML config writing (write_toml_config). Move the existing write_merged_json() and write_toml_config() functions from wm-cli/src/main.rs into this module.
