---
title: WM-000 — Unauthenticated HTTP tool dispatch and wildcard CORS
type: task
status: todo
---


Severity: Critical

Unauthenticated HTTP dispatch of the entire MCP tool registry, with wildcard CORS. Enabler for WM-001 through WM-004 from a browser. Verified live: `access-control-allow-origin: *` returned for `Origin: https://evil.com`.

Per D1 the web UI becomes a read-only viewer, so mutation routes are removed rather than gated.

## Acceptance Criteria

- [ ] `/api/tools/{name}` dispatches only names on an explicit read-only allowlist; all others return 404
- [ ] `POST /api/tools/wm_model` returns 404
- [ ] An allowlisted read-only tool still returns 200
- [ ] `/api/pages/create`, `/api/pages/update`, `/api/pages/delete` are removed from the router and return 404
- [ ] `CorsLayer::permissive()` is removed; no response carries `access-control-allow-origin: *`
- [ ] A request with `Origin: https://evil.com` is rejected
- [ ] Frontend base URLs are relative `/api` in both services
- [ ] `ng serve` still works via the existing `proxy.conf.json`
- [ ] Every `/api/*` route except `/api/health` requires the token from `.wm/state/web-token`
- [ ] `GET /api/health` returns 200 with no token
- [ ] `.wm/state/web-token` is mode 0600 and git-ignored
- [ ] `wm_cli_web_test.rs` passes, with token handling added for non-health routes

## Files

- `apps/wm-server/src/routes/mod.rs` (CORS layer at :72; routes at :57-59)
- `apps/wm-server/src/routes/tools.rs` (:7-17 generic dispatch)
- `apps/wm-server/src/spa.rs` (token injection into index.html)
- `apps/wm-web/src/app/services/http-engine.service.ts` (:7 absolute base)
- `apps/wm-web/src/app/services/http-code-intel.service.ts` (:7 absolute base)
- `apps/wm-core/tests/wm_cli_web_test.rs` (:166, :267, :271, :324 assert health 200)
- `apps/wm-cli/src/main.rs` (:867 readiness probe depends on unauthenticated health)

## Notes

`/api/health` must stay unauthenticated — it gates CLI startup and four existing assertions.
