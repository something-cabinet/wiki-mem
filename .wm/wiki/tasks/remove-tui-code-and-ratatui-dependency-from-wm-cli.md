---
title: Remove TUI code and ratatui dependency from wm-cli
type: task
id: "wiki:tasks:remove-tui-code-and-ratatui-dependency-from-wm-cli"
status: done
priority: high
tags: [from-spec, spec:remove-tui]
acceptance_criteria:
  - text: "AC-1: `wm --tui` is rejected as an unknown flag"
  - text: "AC-2: `wm tui` is not a recognized command (`wm --help` omits it; invoking it errors)"
  - text: "AC-3: `wm` with no args prints help/usage and exits 0 (no TUI launch, no TUI mention)"
  - text: "AC-4: apps/wm-cli/src/tui.rs no longer exists and no `mod tui` / `run_tui` reference remains"
  - text: "AC-5: ratatui is absent from apps/wm-cli/Cargo.toml and from Cargo.lock"
  - text: "AC-6: cargo build and cargo clippy pass with zero warnings"
  - text: "AC-7: wm init and wm setup still show interactive prompts/spinners (dialoguer/indicatif preserved)"
assignee: fixer
---

Delete the ratatui TUI from wm-cli per @wiki/specs/remove-tui. Remove apps/wm-cli/src/tui.rs, the `mod tui` declaration, the `--tui` global flag, the `tui` subcommand variant + dispatch arm, and the auto-launch block in main.rs. Change the no-command path to print help and exit 0. Remove the `ratatui` dependency from Cargo.toml. Keep is-terminal/dialoguer/indicatif (used by wizard flows).