---
title: Missing LICENSE file for a dual MIT OR Apache-2.0 crate, plus wm vs wm-cli naming drift
type: task
id: "wiki:tasks:missing-license-file-and-cli-naming-drift"
status: todo
tags: [docs, packaging, license]
---

Severity: Medium

Two packaging/metadata gaps found during the README fact-check (2026-10-15).

## 1. No LICENSE file

`Cargo.toml` declares `license = "MIT OR Apache-2.0"` but no LICENSE / LICENSE-MIT / LICENSE-APACHE file is committed, and the README previously claimed plain MIT. A dual-licensed crate needs both texts present for the declaration to mean anything.

## 2. CLI identity drift

- npm package: `@something-cabinet/wm-cli`
- shipped binary: `wm-cli`
- embedded MCP server key: `"wm"`
- clap declared name: `"wm"` (`apps/wm-cli/src/shared/models/cli_model.rs:6`), so `--version` and `--help` print `wm` while every doc and config says `wm-cli`

Pick one name and make help output, docs, and configs agree.

## Acceptance Criteria

- [ ] `LICENSE-MIT` and `LICENSE-APACHE` exist at the repo root (or the declared license is changed to a single one)
- [ ] `wm-cli --version` and `wm-cli --help` print the same product name the docs use
- [ ] README License section matches the shipped license files
- [ ] No doc or generated config refers to the tool by a name the binary does not have

## Files

- `Cargo.toml`
- repo root (LICENSE files)
- `apps/wm-cli/src/shared/models/cli_model.rs`
- `apps/wm-core/src/embed_files/configs/dot_mcp.json`
- `README.md`