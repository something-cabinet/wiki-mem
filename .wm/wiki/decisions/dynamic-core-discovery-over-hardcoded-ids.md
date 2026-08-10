---
title: 'Decision: Dynamic Core Discovery Over Hardcoded IDs'
id: wiki:decisions:dynamic-core-discovery-over-hardcoded-ids
type: decision
relates_to:
  - {type: implements, target: wiki:tasks:update-wm-init-skill-for-dynamic-core-page-discovery}
status: approved
tags: [decision, init, core, dynamic-discovery]
---

## Context

The `wm-init` skill previously hardcoded three page IDs (README, ARCHITECTURE, CONVENTIONS) to load at session start. With the introduction of the `core` page type, we needed a way for new core pages to be automatically discovered without requiring a skill file update.

## Decision

Use a hybrid approach in `wm-init`:
1. Always load README explicitly (as the project intro)
2. Dynamically discover all `type: core` pages via `wm_page.list({"type": "core"})` and read each one

## Rationale

- **Future-proof** — adding a new core page auto-loads it at every init without skill edits
- **Backward-compatible** — README still loaded explicitly, no breakage
- **Simple** — uses existing `wm_page.list` tool with a `type` filter, no new infrastructure
- **Discoverable** — the session context summary lists all discovered core pages

## Consequences

- `wm-init` no longer needs updating when new core pages are created
- The `core` type directory convention must be maintained (pages in `core/` with `type: core`)
- Session init may take slightly longer if many core pages exist (mitigated by large-page section-only reading)

## Related
- @wiki/specs:core-page-type — D2: Hybrid init