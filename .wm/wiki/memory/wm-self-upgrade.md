---
title: wm self upgrade
id: wiki:memory:wm-self-upgrade
type: memory
tags: [deployment, npm, removal]
---

REMOVED (2026-07-31) per @wiki/specs/remove-self-install-flow: wm upgrade, wm init --full, and the wm_core::install module (~/.wm/bin copy + PATH registration) no longer exist. Distribution is npm (cargo-npm @something-cabinet/wm-cli) or cargo install. MCP configs always write "wm-cli". Legacy ~/.wm/bin folders are left in place (manual removal).