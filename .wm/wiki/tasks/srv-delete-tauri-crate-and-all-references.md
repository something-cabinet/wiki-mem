---
title: SRV: Delete Tauri crate and all references
type: task
status: todo
priority: high
tags: [spec:wm-server, cleanup]
---

After Angular is wired to HTTP:
1. Delete apps/wm-web/src-tauri/ entirely
2. Remove tauri deps from workspace Cargo.toml
3. Remove Tauri workspace members
4. Remove @tauri-apps/api, @tauri-apps/cli from package.json
5. Remove __TAURI_INTERNALS__ references from Angular code
6. Remove Tauri pilot test runner
7. Verify cargo build passes with zero Tauri references