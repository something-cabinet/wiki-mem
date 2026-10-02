---
title: Dependency triage during subsystem removal
type: memory
tags: [pattern, cleanup, dependencies]
status: active
---

When removing a subsystem, triage each dependency by actual usage: subsystem-only deps get removed, deps shared with surviving features stay. Grep the source for call sites, not just Cargo.toml. Full reference: @wiki/patterns/dependency-triage-during-subsystem-removal