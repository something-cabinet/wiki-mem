---
title: SRV: Wire Angular to HTTP — replace Tauri IPC with fetch
type: task
status: todo
priority: high
tags: [spec:wm-server, angular, migration]
---

Rewrite Angular frontend to use HTTP instead of Tauri IPC:
1. Create proxy.conf.json (proxy /api → :4090)
2. Rewrite api.service.ts: replace tauriInvoke with httpCall using fetch()
3. Rewrite graph-view startLayout(): replace Tauri events with EventSource SSE
4. Remove @tauri-apps/api dependency from package.json