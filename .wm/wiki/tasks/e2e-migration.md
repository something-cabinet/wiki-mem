---
title: Migrate legacy CodeceptJS E2E tests to WDIO
type: task
status: todo
---

apps/wm-web-e2e (CodeceptJS) is still pointing at HTTP mock server. Need to either migrate to WDIO or update to use the mock-server's IPC adapter.

Options:
- Migrate CodeceptJS tests to WDIO in apps/wm-web/e2e
- Or update to use wm-mock-server's IPC adapter

References: @wiki/specs/wm-mock-package.md, @wiki/notes/session-handover-2026-07-17.md