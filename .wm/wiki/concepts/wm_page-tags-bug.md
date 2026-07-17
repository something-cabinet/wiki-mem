---
title: Failure: wm_page.create silently drops tags
type: concept
status: reviewed
tags: [failure, tags, wm_page, bug]
---

## What went wrong
`wm_page.create` accepted `tags` in its JSON schema (generated from the `Create` variant of `WmPageAction`) but the handler matched it as `tags: _` — silently discarding the value. Pages created via `wm_page.create` never had tags in their frontmatter.

## Root cause
The `Create` variant of `WmPageAction` at `mcp/tools/page.rs` used `tags: _` in the pattern match instead of `tags` (which would bind the value and use it). This was present since the typed-page feature was added.

## Time lost
Unknown — could have been months of untagged pages. The bug only surfaced during the retire-wm-doc audit when oracle reviewed feature parity between `wm_doc` and `wm_page`.

## Prevention
- When adding new fields to an action enum variant, always verify they're consumed in the handler, not prefixed with `_`
- Code review for pattern match arms that use `_: Type` — likely indicates a dropped value
- Test: create page with tags, read file from disk, verify tags in frontmatter

## Related
- @wiki/specs/retire-wm-doc
- `apps/wm-core/src/mcp/tools/page.rs`
