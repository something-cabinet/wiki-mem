---
title: "SRV: Wire Angular to HTTP — replace Tauri IPC with fetch"
id: b6aaba
type: task
status: todo
priority: high
tags: [spec:wm-server, angular, migration]
acceptance_criteria:
  - text: "proxy.conf.json created proxying /api to :4090"
  - text: "api.service.ts uses fetch()-based httpCall instead of tauriInvoke, and graph-view startLayout() uses EventSource SSE instead of Tauri events"
  - text: "@tauri-apps/api dependency removed from package.json and the Angular app builds and runs against the HTTP backend"
---

Rewrite Angular frontend to use HTTP instead of Tauri IPC:
1. Create proxy.conf.json (proxy /api → :4090)
2. Rewrite api.service.ts: replace tauriInvoke with httpCall using fetch()
3. Rewrite graph-view startLayout(): replace Tauri events with EventSource SSE
4. Remove @tauri-apps/api dependency from package.json
