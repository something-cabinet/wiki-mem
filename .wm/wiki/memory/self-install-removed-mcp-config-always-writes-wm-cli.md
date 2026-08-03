---
title: Self-install removed — MCP config always writes wm-cli
type: memory
tags: [deployment, npm, install, decision]
status: active
---

WM distribution decision (2026-07-31): self-install (~/.wm/bin + PATH via wm upgrade / wm init --full) is redundant with cargo-npm distribution and broken on macOS (ensure_on_path writes ~/.profile which zsh ignores). Removed entirely. MCP config generation (wm setup opencode) always writes "command": "wm-cli" — the user is assumed to have it on PATH. D1-D4 locked in @wiki/specs/remove-self-install-flow.