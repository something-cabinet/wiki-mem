---
title: 'Failure: HTTP Service Response Envelope Inconsistency'
type: concept
id: wiki:concepts:response-envelope-inconsistency
relates_to:
  - {type: references, target: wiki:decisions:separate-service-ports-over-monolithic-engineport}
---
id: wiki:concepts:response-envelope-inconsistency

---
id: wiki:concepts:response-envelope-inconsistency
title: Failure: HTTP Service Response Envelope Inconsistency
type: concept
tags: [failure, angular, http, api, consistency]
---
id: wiki:concepts:response-envelope-inconsistency

## What went wrong

Two HTTP service implementations in the same project handled the server response envelope differently. `HttpEngineService` returned raw JSON. `HttpCodeIntelService` unwrapped `{success, data}`. The `HttpEngineService` consumers silently read `undefined` data because the server nests page data under `res.page.*` and `res.page.meta.*`, not at `res.*`.

## Root cause

- No documented convention for the HTTP response envelope format.
- `HttpEngineService` was built before the `{success, data}` envelope pattern was established.
- The `httpCall` helper was duplicated across services instead of shared.

## Prevention

- All HTTP service implementations MUST use the same envelope unwrapping: extract `{success, data}`, throw on `!success`, return `data`.
- Extract a shared `httpCall` helper or base class to prevent future divergence.
- When adding new service ports, check existing implementations for the envelope pattern.
- Mock services should produce responses in the same shape as the real server.

## Time lost

~30m Oracle review + fix time.

## Related
- @wiki/decisions:separate-service-ports-over-monolithic-engineport