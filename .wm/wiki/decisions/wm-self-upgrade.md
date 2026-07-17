---
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
