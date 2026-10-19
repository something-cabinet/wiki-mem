---
title: Decide whether PageState's fields and AuditState::started_at are dead or merely unwired
type: task
id: "wiki:tasks:decide-whether-page-state-and-started-at-are-dead"
status: done
tags: [refactor, deadcode, wm-core]
priority: medium
acceptance_criteria:
  - {text: "It is established whether page_contents/content_hashes are genuinely unused or whether a caller is missing", checked: false}
  - {text: "The fields are either deleted (with PageState) or given a real caller; nothing is suppressed with #[allow]", checked: false}
  - {text: "AuditState::started_at is resolved the same way", checked: false}
  - {text: "cargo test --workspace stays green afterwards", checked: false}
implementation_notes: |-
  Resolved by evidence, not judgement.

  `page_contents` and `content_hashes` were never read by any caller in the repository's whole history:
  `git log -S'content_hashes.read'` and `-S'content_hashes.write'` return nothing, and
  `-S'page_contents.get'` / `-S'page_contents.insert'` return only the ES-07 commit that created the handle.
  They were vestigial from birth, not a missing integration, so both were deleted along with PageState.

  `AuditState::started_at` was the same shape - set at construction, read nowhere - and is deleted.

  `WikiPageContent` was NOT deleted: page_crud_service.rs and tools/page/mod.rs still construct it, so the
  dead field was not its only user.
---

**Finding from @wiki/specs:engine-state-decomposition.** Two `EngineState` fields are dead after the
decomposition, and the migration deliberately preserved them rather than deleting them.

`PageState` exposes eight methods over `page_contents` and `content_hashes`, and no product code calls any of
them. Measured 2026-10-19: there is no `engine.page.*` access in `apps/wm-core`, `apps/wm-cli` or
`packages/wm-tools`; the only callers are `page_state_model.rs`'s own tests. `AuditState::started_at()` has
no reader either.

**Why it matters.** @wiki/rules/no-dead-code-clone-scanning says dead code must be removed or restructured,
never suppressed. These fields are not suppressed - they are public API with no caller - but they occupy a
handle and its tests, which makes the handle look load-bearing when it is not.

**Decision needed.** Either delete the two fields and the `PageState` handle with them (which would also retract
ES-07's migration), or give them a real caller. Deleting is the smaller change if page content is genuinely
never cached in memory; that should be confirmed before acting, because a cache that is written but never read
may indicate a missing integration rather than an unused feature.