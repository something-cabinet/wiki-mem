---
title: init-setup-separation ADR updated with path resolution
type: memory
tags: [cli, init, setup, decision]
status: active
---

init-setup-separation ADR now documents two-tier path resolution: init uses canonical wm-cli, setup resolves actual binary path. Init --full chains install → wiki → opencode.json + skills. @wiki/decisions:init-setup-separation