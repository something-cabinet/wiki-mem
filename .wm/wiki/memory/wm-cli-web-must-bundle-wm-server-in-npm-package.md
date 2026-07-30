---
title: wm-cli web must bundle wm-server in npm package
type: memory
tags: [npm, ci, wm-server, wm-cli, deployment]
status: active
---

The `wm web` command resolves `wm-server` via `resolve_server_binary()` with 3-tier priority:
1. Same directory as `wm-cli` binary (works for cargo-built and npm-bundled with both binaries)
2. `WM_SERVER_PATH` env var
3. PATH scan

The cargo-npm config in `wm-cli/Cargo.toml` lists `bins = ["wm-cli", "wm-server"]` — both binaries must be built and placed in target dirs before `cargo npm generate`. The CI workflow handles this in the `publish` and `publish-npm` jobs.