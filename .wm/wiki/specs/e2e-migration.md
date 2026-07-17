---
title: E2E Migration — CodeceptJS to WDIO + Tauri IPC
type: spec
status: draft
---

## E2E Migration Spec

### Goal
Migrate all 14 CodeceptJS E2E journey scenarios to WDIO test files using Tauri IPC browser mode.

### Current State
- **WDIO** (apps/wm-web): wdio.conf.ts configured, 1 test (graph.test.ts) with mockIPC
- **CodeceptJS** (apps/wm-web-e2e): 7 journeys (14 scenarios), 6 page objects, 11 mock mappings, MockManager helper pointing at HTTP mock server (port 8081)

### Migration Plan
Convert each CodeceptJS journey to a WDIO test file in apps/wm-web/e2e/:

| Journey | Scenarios | WDIO File | Priority |
|---------|-----------|-----------|----------|
| navigation | 1 | navigation.test.ts | high |
| search | 3 | search.test.ts | high |
| pages | 3 | pages.test.ts | high |
| tasks | 2 | tasks.test.ts | medium |
| graph | 2 | graph.test.ts (exists) | done |
| settings | 2 | settings.test.ts | medium |
| memory | 3 | memory.test.ts | medium |

### Pattern
Each WDIO test uses \rowser.mockIPC()\ to register Tauri IPC command handlers, mirroring the CodeceptJS MockManager + mappings approach. Page object methods become inline helper functions since WDIO doesn't use CodeceptJS's actor pattern.

### Success Criteria
- All 14 scenarios runnable via \
px wdio run wdio.conf.ts\
- No dependency on HTTP mock server (port 8081)
- CodeceptJS project retained until full migration validated

### References
@wiki/tasks/e2e-migration, @wiki/specs/wm-mock-package