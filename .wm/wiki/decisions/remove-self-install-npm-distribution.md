---
{}
relates_to:
  - {type: references, target: wiki:tasks:remove-self-install-flow-wm-upgrade-install-module---full-flag}
---

---
{}
relates_to:
  - {type: supersedes, target: wiki:decisions:wm-self-upgrade}
---

---
title: Decision: Remove self-install — npm/cargo distribution, deterministic MCP config
type: decision
id: wiki:decisions:remove-self-install-npm-distribution
status: approved
tags: [decision, deployment, npm, mcp]
---

## Context

WM's binary self-install flow (`wm upgrade`, `wm init --full` → copy `current_exe()` to `~/.wm/bin/` + register PATH) was designed to mirror Knowns' `~\.knowns\bin\` pattern, predating the project's actual distribution channel. The cargo-npm pipeline (`@something-cabinet/wm-cli` metapackage + per-platform native binaries + `wm-server` as optionalDependencies, resolved at runtime from `node_modules`) already provides a stable, per-user, PATH-registered install. On macOS the self-install was additionally broken: `ensure_on_path()` appends `export PATH=...` to `~/.profile`, which zsh never sources — so the PATH half was silently inert while leaving a stale ~170MB binary in `~/.wm/bin/`.

## Decision

Remove the self-install flow entirely, all platforms (D1). Distribution is npm (`npm i -g @something-cabinet/wm-cli`) or `cargo install`. Generated MCP configs (e.g. `wm setup opencode`) **always** write `"command": "wm-cli"` unconditionally — no `is_installed()` check, no PATH lookup, no `current_exe()` fallback (D3). The user is assumed to have `wm-cli` on PATH. `wm init --full` is removed; `wm init` is the only init path (D2). Existing `~/.wm/bin/` folders are left in place; removal is manual (D4).

## Rationale

- npm covers all supported platforms including win32-x64 — the self-install was redundant everywhere, not just on macOS
- Deterministic generated configs beat adaptive detection: the same output on every machine, no hidden state, no fragile absolute paths in committed configs
- The macOS PATH registration was broken anyway (zsh ignores `~/.profile`) — the mechanism never fully worked on the primary non-Windows platform
- Two distribution channels doing the same job means double maintenance and confusing failure modes

## Consequences

- `wm upgrade`, `wm init --full`, and the `wm_core::install` module no longer exist
- If `wm-cli` is missing on PATH, MCP startup fails loudly ("command not found") — no silent fallback
- Dev workflow: run from `target/debug/` or ensure `wm-cli` is on PATH
- Supersedes the self-upgrade decision; the two-tier path-resolution strategy in init-setup-separation is obsolete

## Related

- @wiki/tasks/remove-self-install-flow-wm-upgrade-install-module---full-flag
- @wiki/specs/remove-self-install-flow
- @wiki/decisions/wm-self-upgrade (superseded)
- @wiki/decisions/init-setup-separation (path-resolution subsection obsolete)