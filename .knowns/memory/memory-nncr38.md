---
id: nncr38
title: Gitea CI/CD for Rust CLI tools
layer: project
category: pattern
tags:
  - ci
  - gitea
  - rust
  - deployment
createdAt: '2026-07-07T10:34:52.925Z'
updatedAt: '2026-07-07T10:34:52.925Z'
---

Self-hosted Gitea CI for Rust projects: local-host runner, CARGO_TARGET_DIR cache at /home/gitea/actions-cache/cargo/target, smart skip pattern using `git diff --name-only HEAD~1` to skip tests when unrelated files change. Build + test on push to master/dev, release binary on tags via `cargo build --release`. No Docker, no DB needed for CLI/MCP tools. Pattern from gehenna-app.
