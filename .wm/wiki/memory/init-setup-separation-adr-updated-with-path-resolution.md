---
title: init-setup-separation ADR updated with path resolution
type: memory
status: active
tags: [cli, init, setup, decision]
---

init-setup-separation ADR documented two-tier path resolution: init uses canonical wm-cli, setup resolves actual binary path. UPDATE (2026-07-31): self-install flow removed per @wiki/specs/remove-self-install-flow — wm init --full and wm upgrade no longer exist; MCP configs always write "wm-cli".