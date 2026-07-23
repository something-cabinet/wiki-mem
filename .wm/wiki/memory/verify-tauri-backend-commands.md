---
title: "Verify Tauri backend commands exist for all frontend invoke() calls"
type: memory
status: active
tags: [tauri, frontend, api, debugging, verification]
relates_to:
  - {type: references, target: wiki:tasks:review-payload-mapping-api-service}
---

Tauri provides no compile-time check that frontend `invoke('command_name', ...)`
calls match backend `#[tauri::command]` functions. If a command is called from
the frontend but not implemented on the backend, the error is a runtime IPC error.

In this session, `update_page` and `delete_page` were called from the Angular
`ApiService` but had no corresponding Tauri command in `commands.rs`.

**Prevention:** Periodically audit `apps/wm-web/src/app/services/api.service.ts`
against `apps/wm-web/src-tauri/src/commands.rs` to ensure every `tauriCmd()` call
has a matching backend handler registered in `lib.rs`'s `generate_handler![]` macro.

**Reference:**
- `apps/wm-web/src/app/services/api.service.ts`
- `apps/wm-web/src-tauri/src/commands.rs`
- `apps/wm-web/src-tauri/src/lib.rs`
- @wiki/tasks/review-payload-mapping-api-service
