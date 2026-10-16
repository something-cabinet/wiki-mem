---
title: Prove writer parity across revisions, not against the on-disk corpus
type: memory
tags: [refactor, testing, serialization, parity]
status: active
---

When a refactor moves/re-types a persisted-output writer, on-disk byte identity is the wrong oracle if the corpus predates the writer. Build the base revision in a detached worktree, dump both writers' output for the whole corpus over identical input, and diff; keep it as a checked-in script. Full reference: @wiki/patterns/prove-writer-parity-across-revisions