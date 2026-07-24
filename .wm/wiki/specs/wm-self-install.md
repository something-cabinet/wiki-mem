---
id: wiki:specs:wm-self-install
title: "WM Self-Install — Binary Deployment + PATH Registration"
page_type: spec
status: approved

> **Implementation note:** `wm init --full` chains install → PATH → project init. `wm setup install` is standalone. `wm setup opencode` uses PATH-based `wm-cli` command when installed. Implemented in commit TBD.
tags: [spec, install, deploy, setup]
---
id: wiki:specs:wm-self-install

## Overview

WM currently only runs from `target/debug/wm-cli.exe` — the Rust build output. For end-users (and AI agents), WM needs to install itself to a stable location and register on PATH, matching the pattern Knowns uses with `~\.knowns\bin\`.

## Locked Decisions

- **D1 — Install location**: `~\.wm\bin\wm-cli.exe` (mirrors Knowns' `~\.knowns\bin\` pattern). Not versioned — install always replaces the binary.
- **D2 — PATH registration**: User-level PATH via Windows registry `HKCU\Environment`. No admin required. Works immediately on new shells; existing shells need re-login or `refreshenv`.
- **D3 — `wm setup install` command**: New subcommand that copies the running binary and registers PATH. Idempotent.
- **D4 — `wm setup opencode` uses PATH**: After install, `opencode.json` references `wm` instead of the full path to `target/debug/wm-cli.exe`.
- **D5 — Self-executable detection**: The binary detects its own path at runtime via `std::env::current_exe()` to know what to copy.

## Requirements

### FR-1: wm setup install

New CLI subcommand:

```
wm setup install [--prefix ~\.wm]
```

Defaults to `~\.wm`. Steps:
1. Create `~\.wm\bin\` directory
2. Copy `std::env::current_exe()` → `~\.wm\bin\wm-cli.exe`
3. Add `~\.wm\bin\` to user `PATH` via `HKCU\Environment` if not already present
4. Print success message

### FR-2: Idempotent PATH Management

Before adding to PATH, check if the entry already exists (avoid duplicates). Use Windows `REG ADD` / `REG QUERY` for PATH management:

```
REG QUERY HKCU\Environment /v PATH
REG ADD HKCU\Environment /v PATH /t REG_EXPAND_SZ /d "...;%USERPROFILE%\.wm\bin" /f
```

Use the Windows `reg` command via `std::process::Command`. The PATH value is `REG_EXPAND_SZ` type (supports `%VAR%` references).

### FR-3: wm setup opencode — Use PATH

After install, `wm setup opencode` writes:
```json
"wm": {
  "command": "wm-cli",
  "args": ["mcp"],
  "enabled": true,
  "type": "local"
}
```

Instead of:
```json
"wm": {
  "command": ["C:\\Users\\hk\\.kimaki\\..\\target\\debug\\wm-cli.exe", "mcp"],
  "enabled": true,
  "type": "local"
}
```

If `wm setup install` hasn't been run yet, `wm setup opencode` should warn but still work (fall back to the current binary path).

### FR-4: wm setup status — Show Install State

Add a status check showing:
- Where WM is installed (`~\.wm\bin\`)
- Whether `~\.wm\bin\` is on PATH
- Whether the installed version matches the current binary

### FR-5: Cross-Platform Placeholder (Non-Windows)

On Linux/macOS:
- Install to `~/.wm/bin/`
- PATH management via `.bashrc` / `.zshrc` / `.profile`
- A future task; for now, only Windows is implemented.

## Acceptance Criteria

- [ ] AC-1: `wm setup install` copies binary to `~\.wm\bin\wm-cli.exe`
- [ ] AC-2: `~\.wm\bin\` added to user PATH after install (verify with `REG QUERY`)
- [ ] AC-3: Running `wm setup install` twice doesn't duplicate PATH entry
- [ ] AC-4: `wm setup opencode` writes `"command": "wm-cli"` when installed
- [ ] AC-5: `wm setup opencode` falls back to absolute path when not installed (with warning)
- [ ] AC-6: `wm setup status` shows install state
- [ ] AC-7: Binary still compiles on Linux (no Windows-only breakage — use conditional compilation)
- [ ] AC-8: `cargo build` succeeds
- [ ] AC-9: `cargo test` passes

## Technical Breakdown

### Files to change

| File | Change | Lines |
|---|---|---|
| `wm-cli/src/main.rs` | Add `SetupInstall` / `SetupStatus` command variants | +20 |
| `wm-cli/src/install.rs` | New file: `install_binary()`, `ensure_on_path()`, `check_installed()` | +80 |
| `wm-cli/src/setup.rs` (or inline) | Update `setup_opencode()` to prefer PATH when installed | +15 |
| **Total** | | **~115** |

### Windows PATH management (pseudocode)

```rust
#[cfg(windows)]
fn ensure_on_path(install_dir: &Path) -> Result<(), String> {
    let output = std::process::Command::new("REG")
        .args(["QUERY", "HKCU\\Environment", "/v", "PATH"])
        .output()
        .map_err(|e| format!("REG QUERY failed: {}", e))?;

    let path_str = String::from_utf8_lossy(&output.stdout);
    if path_str.contains(install_dir.to_str().unwrap()) {
        return Ok(()); // Already on PATH
    }

    let new_path = format!("{};{}", path_str.trim(), install_dir.display());
    std::process::Command::new("REG")
        .args(["ADD", "HKCU\\Environment", "/v", "PATH", "/t", "REG_EXPAND_SZ", "/d", &new_path, "/f"])
        .status()
        .map_err(|e| format!("REG ADD failed: {}", e))?;

    Ok(())
}

#[cfg(not(windows))]
fn ensure_on_path(_install_dir: &Path) -> Result<(), String> {
    // Placeholder — implement shell rc file update later
    Ok(())
}
```

## Non-Goals

- Version management (only one binary, always overwritten)
- Uninstall command (manual deletion is fine for now)
- Global PATH (user-level only, no admin)
- Linux/macOS shell rc auto-detection (hardcode `~/.profile` for now, full shell detection later)
