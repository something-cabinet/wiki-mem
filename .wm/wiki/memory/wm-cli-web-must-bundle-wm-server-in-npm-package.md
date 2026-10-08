---
title: wm-cli web must bundle wm-server in npm package
type: memory
status: active
tags: [npm, ci, wm-server, wm-cli, deployment]
---

schema_version: 1
state: |-
  The `wm web` command resolves `wm-server` via `resolve_server_binary()` with 4-tier priority:
  1. Same directory as `wm-cli` binary (works for cargo-built installs)
  2. `WM_SERVER_PATH` env var
  3. npm scope sibling — walks up from current_exe() scanning `node_modules/@something-cabinet/wm-server-*/`
  4. PATH scan

  IMPORTANT: `cargo-npm` `bins` only accepts binaries from the SAME crate. Attempting `bins = ["wm-cli", "wm-server"]` fails CI with "unknown bin(s)". Multi-crate distribution requires a SEPARATE npm package per binary (`@something-cabinet/wm-server`) listed as `optionalDependencies` of the main package, plus the scope-scan resolver above.
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
