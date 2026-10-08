---
title: TUI removed from wm-cli (ratatui)
type: memory
tags: [tui, ratatui, cli, cleanup]
status: active
---

schema_version: 1
state: |-
  The ratatui TUI was removed from wm-cli (spec @wiki/specs/remove-tui, approved). Deleted apps/wm-cli/src/tui.rs, the `--tui` global flag, the `tui` subcommand, and the auto-launch path (no-command + terminal). `wm` with no args now prints help and exits 0. `ratatui` dependency dropped from Cargo.toml/Cargo.lock. The dialoguer/indicatif wizard prompts and is-terminal stdin detection were KEPT — they are used by `wm init`/`wm setup`/spinners, independent of the TUI. README updated; tui-polish spec marked superseded, tasks 75k8oh/6lzncr cancelled. Primary interfaces are now CLI commands, wm-server HTTP daemon, and wm-web Angular UI.
questions:
  - id: layer
    type: choice
    instructions: Which memory layer does this entry belong to?
    options:
    - project
    - global
    - session
  - id: store_or_skip
    type: noul
    instructions: This entry is worth storing as durable memory.
  - id: dedup_action
    type: choice
    instructions: How should this entry relate to existing memory?
    options:
    - new
    - merge
    - supersede
    - skip
  - id: confidence
    type: score
    instructions: How confident is the recorded knowledge?
    levels:
    - low
    - medium
    - high
answers: {}
