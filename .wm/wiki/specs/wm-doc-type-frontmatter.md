---
title: wm_doc Type Frontmatter Fix
type: spec
id: wiki:specs:wm-doc-type-frontmatter
status: approved
tags:
- spec
- bugfix
- wm-doc
- frontmatter
- mcp
- lint-policy
- approved
relates_to:
  - {type: relates_to, target: wiki:tasks:deadcode-ban-enforcement}
---

# wm_doc Type Frontmatter Fix

## Overview

`wm_doc.create` / `wm_doc.update` drop the `type` field from YAML frontmatter even though it is a whitelisted, API-writable field (GitHub issue #126). Pages still register under the correct `wiki:<type>:` path, but the on-disk frontmatter is incomplete — breaking tooling that reads the first frontmatter block and `wm_validate.check` on short entity ids until an index rebuild. `wm_page.create` persists `type` correctly; the bug is specific to the `wm_doc.*` family. This spec fixes `wm_doc` to full parity with `wm_page` and bans the `#[allow(dead_code)]` pattern that let the bug ship silently.

## Locked Decisions

- D1: Full parity with `wm_page` — wire `type` into `wm_doc.create` and add `type` + `tags` to `wm_doc.update`, with regression tests for both.
- D2: Out of scope: the `wm_validate.check` short-id resolution behavior and the documented `wm-frontmatter-whitelist-limitation` concept — this spec fixes the dropped-field bug only.
- D3: `#[allow(dead_code)]` is banned completely (repo-wide, enforced in CI). The modern `#[expect(dead_code)]` is the sanctioned replacement where dead code is genuinely transient — it errors when the item is actually used, so it self-removes and can never mask a live field.

## Root Cause (investigated — apps/wm-core/src/mcp/tools/doc.rs)

1. **Create ignores `type` by design**: the schema declares `r#type: Option<String>` (doc.rs:35-36) tagged `#[allow(dead_code)]`, the handler destructures it as `r#type: _` (doc.rs:157), and `build_markdown(&title, &content, &tags)` (doc.rs:187) never receives it. The `dead_code` annotation shows the field was declared for API compatibility but never wired — the annotation is what silenced the compiler warning that should have caught this.
2. **Update cannot set `type` at all**: `WmDocAction::Update` accepts only `path/title/content` (doc.rs:40-44); it preserves the existing frontmatter map via `build_markdown_from_map` (doc.rs:241) and can never add or change `type`.
3. **Contrast**: `wm_page` (apps/wm-core/src/mcp/tools/page/action.rs) wires `r#type` through to frontmatter on both create and update — hence the reported asymmetry.
4. **Why tests missed it**: `wm_doc` has no functional test coverage — the only `wm_doc` reference in the test suite is path-confinement in `security_test.rs`. The frontmatter round-trip tests (`mcp_test.rs:217` `page_create_emits_id_frontmatter`, `mcp_test.rs:249` `page_update_extra_frontmatter_persists`) cover only `wm_page`, the family that works. No parity contract exists between the two families.

Not a serialization bug — a dead input field on create plus a missing input field on update, masked by `#[allow(dead_code)]` and by the absence of `wm_doc` frontmatter tests.

## Requirements

### Functional Requirements

- FR-1: `wm_doc.create({ path, title, type, content, tags })` persists `type: <value>` into the YAML frontmatter exactly as `wm_page.create` does; `build_markdown` must receive and serialize the type.
- FR-2: `wm_doc.update` accepts `type` and `tags` (additive to the existing `path/title/content` surface) and persists changes to those fields while preserving all other frontmatter fields.
- FR-3: When `type` is absent, `wm_doc.create` behavior matches `wm_page`'s default (path-derived type) rather than writing an untyped page.
- FR-4: `wm_page.create` / `wm_page.update` behavior is unchanged (no regression from any shared helper refactor).
- FR-5 (dead_code ban): `#[allow(dead_code)]` is removed from `doc.rs` and banned repo-wide. The six `#[allow(dead_code, reason = …)]` in `apps/wm-core/tests/helpers/http_daemon.rs` are converted to `#[expect(dead_code)]`. No new `#[allow(dead_code)]` may be introduced anywhere.
- FR-6 (enforcement): a deterministic CI check in `.github/workflows/ci.yml` (check job) fails the build if `#[allow(dead_code)]` appears anywhere in the repo (excluding `target/`).

### Non-Functional Requirements

- NFR-1: No index rebuild required for the persisted frontmatter to be complete — the fix is at file-write time.
- NFR-2: Backward compatible: no breaking changes to the `wm_doc` tool schema (additive fields only).
- NFR-3: Enforcement is deterministic and dependency-free (grep-based CI step; verified: clippy has no attribute-ban lint — `clippy::disallowed_attrs` does not exist, `allow_attributes_without_reason` only requires reasons).

## Acceptance Criteria

- [ ] AC-1: Regression test: `wm_doc.create({ path: "specs/repro-x", title: "X", type: "spec", content: "..." })` writes `.wm/wiki/specs/repro-x.md` whose frontmatter contains `type: spec`.
- [ ] AC-2: Regression test: `wm_doc.update({ path: "specs/repro-x", type: "howto" })` rewrites frontmatter to `type: howto` and preserves the existing `title` and body.
- [ ] AC-3: Regression test: `wm_doc.update({ path: "specs/repro-x", tags: ["a", "b"] })` persists tags; existing `type` is preserved when not provided.
- [ ] AC-4: Parity test: `wm_doc.create` and `wm_page.create` with identical inputs produce byte-identical frontmatter for `type`.
- [ ] AC-5: Existing `wm_page`, `wm_doc`, and MCP suite tests pass unchanged (no regression).
- [ ] AC-6: `cargo clippy -p wm-core -- -D warnings` clean, and `rg "allow\(dead_code\)"` returns nothing in the repo (doc.rs annotation removed, http_daemon.rs converted to `#[expect]`, CI check in place).
- [ ] AC-7: The CI check step fails the build when a `#[allow(dead_code)]` is introduced (verified by the check's own existence and a manual `rg` run).

## Scenarios

### Scenario 1: Agent creates a typed doc (happy path)
**Given** an agent calls `wm_doc.create({ path: "specs/foo", title: "Foo", type: "spec" })`
**When** the file is written
**Then** `.wm/wiki/specs/foo.md` frontmatter contains both `title: Foo` and `type: spec`.

### Scenario 2: Agent retypes an existing doc
**Given** an existing doc with `type: concept` frontmatter
**When** the agent calls `wm_doc.update({ path: "concepts/foo", type: "howto" })`
**Then** the frontmatter becomes `type: howto`, and all other frontmatter fields and the body are unchanged.

### Scenario 3: Parity with wm_page
**Given** identical create inputs
**When** both `wm_doc.create` and `wm_page.create` run
**Then** the written frontmatter is identical (type included).

### Scenario 4: The ban works (prevention)
**Given** a developer adds `#[allow(dead_code)]` to any file
**When** CI runs
**Then** the check job fails with a reference to the banned annotation, and the developer must use `#[expect(dead_code)]` (self-cleaning) or remove the dead code.

## Technical Notes

- Fix location: apps/wm-core/src/mcp/tools/doc.rs — wire `r#type` through the `Create` handler into `build_markdown` (doc.rs:187), extend `Update`'s input struct (doc.rs:40-44) and handler (doc.rs:201-252), and prefer the same frontmatter-serialization helper `wm_page` uses so the two families cannot diverge again.
- Regression tests belong in apps/wm-core/tests/mcp_test.rs or a new doc-focused test following the inproc harness pattern (tests/helpers/inproc.rs), matching how wm_page type persistence is already tested.
- dead_code ban mechanics: (a) remove the doc.rs annotation as part of the fix; (b) convert http_daemon.rs `#[allow(dead_code, reason = …)]` → `#[expect(dead_code)]` (same reason kept, still self-cleaning); (c) add CI grep step `rg -n "allow\(dead_code\)" --glob '!target/**' && exit 1` (or equivalent) to the check job.
- Reference: GitHub issue #126 (something-cabinet/wiki-mem).

## Open Questions

- [ ] Should `wm_doc` reuse `wm_page`'s create/update implementation directly (full delegation), or stay a thin parallel implementation that shares only the frontmatter builder? (Delegation is maximal parity; shared builder is the lighter touch — planning's call.)
- [ ] Does `wm_doc.get` output need the `type` field surfaced (it currently returns raw frontmatter, so this is likely already covered)?
- [ ] `#[expect(dead_code)]` requires rustc 1.81+ — confirm the repo's MSRV/toolchain supports it (clippy runs on the current toolchain; if the project pins an older MSRV this needs revisiting).