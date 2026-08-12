---
title: No Compensating Layers — Fix the Layer That's Wrong
type: rule
id: wiki:rules:no-compensating-layers
status: active
tags:
- rule
- architecture
- simplicity
- honesty
- testing
relates_to:
  - {type: references, target: wiki:tasks:linus-core-simplicity-rule}
---

## Rule: No Compensating Layers

Every layer of a system must justify its existence directly. A layer whose only job is to compensate for a bug, a limitation, or a lie in another layer is a **bug report**, not a feature — fix the underlying layer. Never build a distributed system to call your own functions.

### Why

On 2026-08-12 an architecture review found "layers of stupidity, each compensating the other": a local single-user tool where the CLI became an HTTP client to its own daemon, tokens guarded a loopback pipe only the same user could reach, a retry loop existed because the daemon invalidated its own credential, and an SSE endpoint returning `{"events": []}` was the stated justification for the whole proxy. Each layer existed to patch the previous layer's mistake. ~5,300 lines of compensating machinery were deleted — port conflicts, 401 retries, readiness races, and stale graphs all died with it. The fix for the deepest bug (a file watcher) had been written all along; the layers existed because the right layer was never wired.

### Requirements

1. **Fix the layer, don't route around it.** If layer B exists to compensate for a defect in layer A, fix layer A. A workaround that cannot be converted into a fix is a bug.
2. **A local single-user tool must not become a distributed system to call its own functions.** No HTTP-to-self, no server spawned so a CLI can `fetch()` its own library, no tokens securing doors only you can open, no retry loops because your own process invalidates your own credential, no hand-rolled protocol implementations where a library exists.
3. **Honesty.** Logs and status lines must reflect reality (do not print "started" before the bind succeeds). A stated justification must be true — a stub returning an empty result is not a justification. Exit codes must propagate failures.
4. **Tests test behavior, not plumbing.** A test that cannot fail (env-gated, `#[ignore]`d wall-clock asserts, key-existence-only checks) is worthless — delete it. A test that can only fail by hanging is broken — enforce per-test timeouts. One daemon per test binary, never per test. Teardown must not require `kill -9`.
5. **Dead things die.** "Remove after diagnosis" comments must be honored. Zero-caller code must be deleted, not preserved. Superseded decisions must be marked, not silently reverted.

### Enforcement

```bash
# self-HTTP: a tool calling itself over the network
rg 'localhost|127\.0\.0\.1' apps/*/src -g '*.rs'
# dead claims and parked diagnostics
rg 'remove after diagnosis|temporarily' apps/ .github/ -g '*.rs' -g '*.yml'
# un-failable tests
rg 'if std::env::var\("TEST_' apps/ -g '*.rs'
rg '#\[ignore\]' apps/ -g '*.rs'
# per-test daemon spawning
rg 'spawn.*(daemon|server)' apps/*/tests -g '*.rs'
# token theater (everything except the web surface)
rg 'token' apps/*/src -g '*.rs' | grep -v web_token
```

Code review must ask, for every layer: **"what is this compensating for?"** If the answer is another layer's bug, the finding belongs in the underlying layer.

### Exceptions

- The web UI needs an HTTP server — browsers genuinely cannot call Rust in-process. That server is justified by a real client, never by the CLI.
- WASM for pure compute (browser-side layout/rerank) — chatty operations that are genuinely client-side.
- CI-only mitigations must carry a removal condition, honored like any deadline.

### Related

- @wiki/rules/no-warnings — suppress nothing, fix everything
- @wiki/rules/no-dead-code-clone-scanning — dead code must be removed, never suppressed
- @wiki/rules/tdd-red-green-refactor — a test that passes before implementation is worthless
- @wiki/rules/findings-first-task-spec — findings become tasks, not ad-hoc patches
- @wiki/core/critical-patterns — verify tree before re-dispatching a failed lane