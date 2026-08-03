---
status: archived
implementation_notes: SUPERSEDED by @wiki/specs/remove-self-install-flow (approved 2026-07-31). Self-deployment via wm upgrade / ~/.wm/bin is removed; cargo-npm distribution replaces it.
relates_to:
  - {type: superseded_by, target: wiki:specs:remove-self-install-flow}
---

---
id: wiki:decisions:wm-self-upgrade
title: "Decision: Binary Self-Deployment via wm upgrade"
type: decision
status: approved
tags: [decision, deployment, self-upgrade, path]
decision:
  context: "WM's MCP config currently references target/debug/wm-cli.exe — a fragile path that breaks after cargo clean. Knowns solves this by copying its binary to ~\.knowns\bin\ and registering on PATH."
  options:
    - "Keep referencing target/debug/ (dev-only, breaks on clean)"
    - "Copy binary to ~/.wm/bin/ and register on PATH (matches Knowns pattern)"
  rationale: "Chosen ~\.wm\bin\ with PATH registration via REG ADD HKCU\Environment. The running binary copies itself — no installer needed. wm init --full chains upgrade → config → project init for one-command setup."
  outcome: "packages/wm-install with install_binary(), ensure_on_path(). CLI commands: wm upgrade (standalone), wm init --full (chained). opencode config uses 'command': 'wm-cli' when installed."
---
id: wiki:decisions:wm-self-upgrade