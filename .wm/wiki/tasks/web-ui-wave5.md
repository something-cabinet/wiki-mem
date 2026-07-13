---
title: Web UI: Backend hardening — audit, CORS, caching
page_type: task
status: todo
priority: medium
tags:
  - web-ui
  - rust
---
# Web UI: Backend hardening — audit, CORS, caching

Wire audit events, restrict CORS, fix cache headers, logging.

## Acceptance Criteria

- [ ] Web mutations write audit events to .wm/log.jsonl
- [ ] CORS restricted to localhost in production
- [ ] Cache-Control: immutable on hashed assets
- [ ] TraceLayer middleware for request logging
- [ ] Standard error response format across all endpoints