---
title: Decision — Per-launch token file for local daemon authentication
type: decision
id: wiki:decisions:local-daemon-auth-token-file
status: draft
---

## Context

The HTTP server (`wm web`) runs on localhost:4090. Any web page the user visits can reach it via `fetch()`. `Access-Control-Allow-Origin: *` grants the attacker page read access to response bodies — so this isn't blind CSRF, it's full data exfiltration.

Binding to 127.0.0.1 does NOT help because browsers reach loopback without restriction. `Origin`/`Sec-Fetch-Site` rejection handles the browser attacker. But a local process (curl, another tool) sends no Origin header at all — only a shared secret stops that.

## Decision

Generate a random token at startup, persist to `.wm/state/web-token` (mode 0600), inject into `index.html` via a meta tag, and require it as an `x-wm-token` header on every request except `/api/health`.

NOT a query parameter (leaks via Referer, shell history, trace logs).
NOT printed to stdout (leaks to terminal scrollback and CI logs).
NOT a static credential (fixed tokens get committed and shared).

## Rationale

Three layers are needed because they defeat different attackers:
1. Same-origin (no CORS) → defeats browser
2. Token → defeats local process
3. The token is only secret BECAUSE permissive CORS is gone (otherwise a page could fetch index.html and read it)

`/api/health` must be exempt: the CLI uses it as a readiness probe (`main.rs:867`), and 4 existing tests assert 200 without auth.

## Consequences

Scripts that previously `curl`'d the API need to read `.wm/state/web-token` first. The Angular frontend reads it from a meta tag at load time.

## Related

- wiki:specs:security-remediation
