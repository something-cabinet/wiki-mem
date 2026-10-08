---
title: 'SRV: Wire Angular to HTTP — replace Tauri IPC with fetch'
id: b6aaba
type: task
status: done
priority: high
tags:
- spec:wm-server
- angular
- migration
acceptance_criteria:
- text: proxy.conf.json created proxying /api to :4090
- text: api.service.ts uses fetch()-based httpCall instead of tauriInvoke, and graph-view startLayout() uses EventSource SSE instead of Tauri events
- text: '@tauri-apps/api dependency removed from package.json and the Angular app builds and runs against the HTTP backend'
---

schema_version: 1
state: |-
  Rewrite Angular frontend to use HTTP instead of Tauri IPC:
  1. Create proxy.conf.json (proxy /api → :4090)
  2. Rewrite api.service.ts: replace tauriInvoke with httpCall using fetch()
  3. Rewrite graph-view startLayout(): replace Tauri events with EventSource SSE
  4. Remove @tauri-apps/api dependency from package.json
questions:
  - id: work_kind
    type: choice
    instructions: What kind of work is this task?
    options:
    - feature
    - bugfix
    - refactor
    - docs
    - test
    - chore
    - migration
  - id: priority
    type: choice
    instructions: What priority is this task?
    options:
    - low
    - medium
    - high
    - urgent
  - id: needs_spec
    type: noul
    instructions: This task depends on a spec.
  - id: has_ac
    type: noul
    instructions: This task has at least one acceptance criterion.
answers: {}
