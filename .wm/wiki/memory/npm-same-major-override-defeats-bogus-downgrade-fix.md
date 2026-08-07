---
title: npm same-major override defeats bogus downgrade fix
type: memory
tags: [npm, audit, security, overrides]
status: active
---

When npm audit's only fix is a bogus downgrade of a direct dependency (exact-pinned vulnerable transitive, isSemVerMajor pointing at older version), use a same-major overrides entry to force the patched version. Can't override a direct dep unless specs match (EOVERRIDE) — bump the direct range instead. Major-jump overrides need real-suite verification, not just a build. Full reference: @wiki/patterns/npm-override-vs-bogus-downgrade