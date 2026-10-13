---
title: Verify durable writes by read-back before reporting success
type: memory
tags: [tool-reliability, persistence, testing, pattern]
status: active
---

A write helper returning Ok is not proof of persistence. Re-read the file from disk, parse it, and assert the specific field changed before returning success; otherwise error. Order derived stores (version/index) after the source-of-truth write. Regression tests must assert the artifact on disk, not the response and not pre-existing fixture content. Full reference: @wiki/patterns/verify-writes-by-read-back