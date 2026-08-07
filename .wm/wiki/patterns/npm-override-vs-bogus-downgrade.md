---
title: 'Pattern: Same-Major Override vs Bogus npm Audit Downgrade'
type: pattern
id: wiki:patterns:npm-override-vs-bogus-downgrade
status: reviewed
tags:
- pattern
- npm
- audit
- security
- overrides
- dependencies
relates_to:
  - {type: references, target: wiki:tasks:bump-codeceptjs-36--4x-in-wm-web-e2e-dependabot}
---

# Pattern: Same-Major Override vs Bogus npm Audit Downgrade

> Type: pattern | Tags: [pattern, npm, audit, security, overrides, dependencies]

## Problem

`npm audit` flags a vulnerable transitive dependency (e.g. axios 1.16.1, serialize-javascript 6.x, undici 5.x) that a direct dependency pins with an **exact version** (e.g. codeceptjs 4.1.0 pins `axios@1.16.1`). npm's only auto-fix path is a nonsense **downgrade** of the direct dependency (`npm audit fix` → "will install codeceptjs@3.5.4, which is a breaking change"), which would undo the work and break the tree. The patched version exists but npm won't reach it because the pin is exact.

## Solution

Use an npm `overrides` entry to force the patched version, staying within the **same major** so it's runtime-compatible:

```json
{
  "overrides": {
    "axios": "^1.18.0"
  }
}
```

Key rules learned:
- **Same-major only** = safe. If the patched version requires a major jump (serialize-javascript 6→7, undici 5→7), the override is still usually runtime-safe for *utility* packages (single serialization/fetch functions), but must be verified with the real test suite, not just a build — the override sits inside the consumer's runtime tree.
- **Direct-dependency conflict (EOVERRIDE)**: you cannot override a package that is ALSO a direct dependency unless the override spec matches the direct spec exactly. Fix by bumping the direct dependency's range in `package.json` instead, then overriding transitives only.
- **Scoped override** works when the same package appears at multiple versions: `"overrides": { "parent": { "child": "^patched" } }`.
- **Verify, don't assume**: after any major-jump override, run `codeceptjs check` + `dry-run` + the full headless suite (or equivalent smoke for the consuming tool) — a passing build alone doesn't prove the overridden dependency works at runtime.

## When to Use

- `npm audit`'s only suggested fix is a downgrade of a direct dependency (`isSemVerMajor: true` pointing at an OLDER version)
- A transitive dependency is pinned exact by a library that hasn't caught up to the advisory fix
- You want to clear high-severity advisories without waiting for upstream

## When Not to Use

- When the fix requires a major version of a package with a rich API surface your code touches directly (framework internals, plugin APIs) — an override could silently change behavior
- When the vulnerable path is dev-only tooling and the advisory is low/moderate — evaluate risk vs. forced-version risk
- When the direct dependency's OWN update to a patched version is imminent — prefer the real fix

## Related

- Applied this session: axios ^1.18.0, serialize-javascript ^7.0.5, undici ^7.29.0 overrides in wm-web-e2e (codeceptjs 4.1.0 exact pins) — all verified with the 26/26 headless e2e suite