---
title: Create-path record conformance — one builder for page, task and memory
type: spec
id: "wiki:specs:create-path-record-conformance"
status: draft
tags: [spec, write-path, record-format, mcp, tool-reliability]
---

schema_version: 1
state: |-
  ## Overview

  Every page-creation path in `wm` forms its own frontmatter and body. Three defects fall out of that duplication, all in the create flow:

  - **D1** `wm_page.create` silently ignores `extra_frontmatter` for every page type, so acceptance criteria and priority cannot be set at creation time.
  - **D2** `create` writes a **prose** body on record-bearing types, so pages are born non-conformant and the tool can only warn about it afterwards.
  - **D3** `wm_task.create` hand-forms YAML (`format!("  - text: \"{}\"")`, escaping only `"`), so a title or AC containing a colon, a newline or a leading `[` produces a broken document.

  This spec introduces **one create-path builder** used by page, task and memory creation: it applies extra keys through the YAML-aware setter, routes record-bearing types through the existing record-body builder, and never hand-forms YAML.

  ## Evidence

  **D1 - verified by controlled probe (2026-10-16), four cases on scratch pages, all deleted afterwards:**

  | Case | Call | Result |
  |---|---|---|
  | A | `update` with `content` **and** `extra_frontmatter` | ACs **present** |
  | B | `update` with `extra_frontmatter` alone | ACs **present** |
  | C | `create` (spec) with `extra_frontmatter` | ACs **missing** |
  | D | `create` (task) with `priority` + `acceptance_criteria` | both **missing** |

  The discriminator is **create vs update**, not the presence of `content`. `wm_page.update` is correct: `page_update_builder_service.rs:206` applies `extra_frontmatter` unconditionally, and `set_yaml_value_field` appends a key that is missing (`page/helpers/yaml_helper.rs:447-449`). The `Create` arm (`apps/wm-core/src/mcp/tools/page/mod.rs:200-205`) never reads the field - compare the `Update` arm, which threads it into `PageUpdateParams`.

  An earlier report on `wiki:tasks:wiki-tool-reliability-wmpageupdate-ignores-extrafrontmatter-with-content` attributed this to content-presence and to the `update` path. That was wrong; the task has been corrected and retitled.

  **D2 - verified by reading the create path.** `page/mod.rs:203` calls `page::create_page(engine, &path, &frontmatter, &content)` with the raw content, then **computes** `record_format_hint` (line 203) and returns it as `format_warning`. The tool therefore detects non-conformance and writes it anyway. Meanwhile `build_record_body(page_type, state)` already exists (`page/services/record_migration/record_body_builder.rs:13`, re-exported at `record_migration/mod.rs:19`) and is already called by `mcp/tools/decision.rs:135` and by the migration planner - so the builder is proven and unused by the very paths that need it.

  **D3 - verified by reading `mcp/tools/task/mod.rs` (`handle_create`).**

  ```rust
  frontmatter.push_str("acceptance_criteria:\n");
  frontmatter.push_str(&format!("  - text: \"{}\"\n", ac.replace('"', "\\\"")));
  ```

  Escaping only the double quote is insufficient for YAML: a newline, a `: ` sequence, or a value starting with `[` all break the mapping. This is the same root as three open tasks: `frontmatter-serializer-quoting-for-task-titles-and-acs`, `wmpageupdate-corrupts-frontmatter-on-non-indented-yaml-list-fields`, and `same-latent-block-consumption-pattern-in-removeyamlblock-setyamlvaluefield-and-setyamlblock`.

  **Consequences observed:** every task and spec authored through the API in the 2026-10-15 session needed a two-call workaround (create, then a second `update` with `extra_frontmatter` alone); four pre-existing tasks carry no acceptance criteria at all; and a parallel worker filed the equivalent defect for `wm_memory.add`.

  ## Decision

  - **D-1 - Three separate services; one shared low-level helper.** `wm_page.create`, `wm_task.create` and `wm_memory.add` remain **three separate services**, each in its own module (`mcp/tools/page`, `mcp/tools/task`, `mcp/tools/memory`), each keeping its own handler, parameter validation and response shape. What they share is **only the low-level construction step**: one helper in `page/services/` that turns (page type, prose, extra keys) into a conformant page. No service assembles frontmatter or a record body by hand. A helper needed by three callers belongs at a shared level per CONVENTIONS' shared-code rule, not inside one of them.
  - **Wording corrected 2026-10-16.** D-1 previously read "One create-path builder", which was reasonably taken to mean the three services were being merged. They are not. The three handlers stay separate; only construction is common.
  - **D-2 - Record bodies are built, not accepted.** For record-bearing types the builder produces the v1 record body via the existing `build_record_body`. The caller's prose becomes the `state` payload.
  - **D-3 - Existing envelopes pass through untouched.** If the supplied content already parses as a record body for that type, the builder uses it verbatim rather than wrapping it again. This preserves the documented authoring workaround (`wiki:howto:author-record-pages-with-an-explicit-envelope`) and avoids double-wrapping for callers that already build envelopes.
  - **D-4 - `extra_frontmatter` is applied on create**, through the same apply logic the update path uses - one shared function, not a second copy.
  - **D-5 - Hand-formed YAML is deleted** from the task create path; all scalar and list emission goes through the existing YAML-aware helpers.
  - **D-6 - Unknown or unsupported extra keys are an explicit error**, not a silent discard. The rule stands: if the schema accepts it, the handler must consume it.
  - **D-7 - Non-record types keep today's behaviour.** The change is scoped to record-bearing types plus the extra-key application.
  - **D-8 - `format_warning` is retained** as a warning for pages that already exist on disk, but a create through this builder must never produce a page it would warn about.

  ### Rejected alternatives

  - **Patch only the `Create` arm to apply extra keys.** Closes D1, leaves prose bodies and hand-formed YAML - and leaves three ad-hoc create paths in place, which is the actual defect.
  - **Convert prose to record lazily on read.** Masks the write path and makes every reader pay. The write path is where the invariant belongs.
  - **Fix the quoting in place in `handle_create`.** Re-implements YAML escaping; the duplication is the bug, not the symptom.
  - **Make `content` record-only and reject prose outright.** Breaks every existing caller and the standing `migrate-records` workflow, for no gain over pass-through (D-3).

  ## Contract

  - **No breaking API change.** Every previously accepted parameter keeps its meaning. One **additive** change is both required and expected: `wm_page.create` gains `extra_frontmatter`, which it never declared and therefore discarded silently (probe case C). Without accepting that parameter, AC-1 is unsatisfiable. The earlier wording here read "No API change"; additive is the accurate claim, and the implementation disclosure matches it.

  - **Implemented 2026-10-16.** Builder at `apps/wm-core/src/page/services/page_create_builder_service.rs`; `page/mod.rs:203`, `task/mod.rs:385` and `:871`, `memory.rs:361` all route through it. Independently verified: `cargo test -p wm-core --test mcp_test` -> 99 passed / 0 failed; clippy reports 3 warnings, all in files this change did not touch; the 8 adapted existing tests were audited and each relocation preserves the original assertion's intent.
  - `create(type=T, content=S, extra=E)`:
    - if `S` parses as a record body for `T` -> use `S` verbatim (D-3);
    - else if `T` is record-bearing -> body = `build_record_body(T, S)` (D-2);
    - else -> body = `S` (D-7);
    - then apply `E` via the shared YAML-aware apply function (D-4), erroring on an unsupported key (D-6).
  - `wm_task.create` with `acceptance_criteria` emits those criteria through the same path - no bespoke escaping.

  ## Change Inventory

  - **New:** a create-path builder service, following the repo's role-based file naming (`*_builder_service.rs`, one type per file, re-exported through the directory barrel).
  - **`apps/wm-core/src/mcp/tools/page/mod.rs`** - the `Create` arm calls the builder instead of `build_frontmatter` + `create_page` directly.
  - **`apps/wm-core/src/mcp/tools/task/mod.rs`** - `handle_create` drops its hand-rolled frontmatter and AC emission and calls the builder.
  - **`apps/wm-core/src/mcp/tools/memory.rs`** - the `Add` path calls the builder, closing the equivalent defect for memory.
  - **Reused, not reimplemented:** `build_record_body` (`record_body_builder.rs`), the YAML-aware setters (`page/helpers/yaml_helper.rs`), and the update path's extra-key apply logic (extracted so both paths share one copy).
  - **Tests** in the MCP test suite.

  ## Migration

  None. Existing prose pages stay as they are; they are already covered by `wiki:tasks:fix-pre-existing-wiki-frontmatter-parse-errors` and the `wm page migrate-records` command. Nothing already on disk is rewritten by this change.

  ## Acceptance Criteria

  - [ ] AC-1: `wm_page.create` applies `extra_frontmatter` for every record-bearing page type; regression tests reproduce probe cases C and D and assert the keys are present on read-back.
  - [ ] AC-2: `create` on a record-bearing type emits a body containing `schema_version`, `state`, `questions` and `answers` - never prose.
  - [ ] AC-3: A task created with `acceptance_criteria` whose text contains `"`, `: `, a newline and a leading `[` round-trips intact and the page parses.
  - [ ] AC-4: `wm_memory.add` emits a conformant record body.
  - [ ] AC-5: Exactly one implementation of record-body construction exists - `rg` gate confirms the builder has no second copy, and `build_record_body` is called from the create paths.
  - [ ] AC-6: No hand-formed frontmatter string concatenation remains in any create path.
  - [ ] AC-7: Create and update share one function for applying extra keys - no duplicated apply logic.
  - [ ] AC-8: Supplying an already-valid record envelope passes through unmodified (no double-wrapping).
  - [ ] AC-9: A value that cannot be rendered into the frontmatter returns an explicit error rather than being discarded silently. **Scope note (2026-10-16, after recon):** key-*level* validation - rejecting a key that is not valid for its page type - requires the per-type schema that `wiki:tasks:per-type-schema-enforcement--requiredoptional-typed-frontmatter-keys--mandatory-acs-for-task-and-spec` introduces. `packages/wm-engine/src/models/page_type_model.rs` exposes only `allowed_statuses()`, with no allowed-keys list, so that check cannot be built here without inventing one. Since that task already `depends_on` this one, key-schema validation lands there.
  - [ ] AC-10: Existing suite green; `cargo build` and `cargo clippy` warning-free; no `#[allow(dead_code)]`.
  - [ ] AC-11: A task created through the API with ACs no longer trips `wm_validate.check`'s missing-AC rule.

  ## Verification

  1. Regression tests for the four probe cases from the Evidence table, asserting by read-back from disk.
  2. The quoting cases from AC-3.
  3. Envelope pass-through (AC-8) and unsupported-key error (AC-9).
  4. `wm_validate.check` before and after, showing the missing-AC count is unchanged for pre-existing pages and zero for newly created ones.
  5. Read-back of every edited wiki page to confirm frontmatter and record body survived (rule `durable-write-or-error`).

  ## Risks

  - **Double-wrapping** is the main hazard: many callers - including this session's own workflow - pass a full envelope as `content`. D-3 exists specifically to prevent that; AC-8 is its guard.
  - **Blast radius:** the create path is used by every authoring flow, so a regression here is broad. The existing suite plus the new regression tests are the mitigation.
  - **Ordering:** `wiki:tasks:per-type-schema-enforcement--requiredoptional-typed-frontmatter-keys--mandatory-acs-for-task-and-spec` `depends_on` the D1 fix - enforcing ACs before this lands would reject callers who did supply them. See @wiki/patterns:order-guards-after-their-enabling-fix.
  - **Coordination:** a parallel session has filed the equivalent memory defect and is active in the same files; land this before it duplicates the work, or coordinate.

  ## Rollback

  Single-path change behind one builder: reverting the create arms to their previous bodies restores prior behaviour, and no on-disk data is rewritten. Nothing here is irreversible.

  ## Amendment (2026-10-16) - wm init must seed conformant starter pages

  Owner decision: `wm init` should create the core starter pages, through the same shared builder rather than by direct file writes.

  **Why.** `scaffold_project` (`apps/wm-cli/src/init/init_service.rs:164`) writes `config.json`, `.wm/AGENTS.md` and the root `WIKI-MEM.md` shim; `create_wiki_dirs` (`:188-192`) calls `create_dir_ALL` per subdirectory and nothing else. **`wm init` writes no wiki pages at all.** Yet the shim it does write instructs every agent to read `core/README`, `core/ARCHITECTURE`, `core/CONVENTIONS`, `core/critical-patterns` and every active page under `rules/` at session start. On a freshly initialised project none of those pages exist, so `wm-init` loads an empty wiki and reports success.

  **Scope.** The four core pages above, seeded **only when absent**, so an existing wiki is never clobbered. Each is scaffolding *for the user's project* with clearly-marked placeholder sections - explicitly **not** a copy of this repository's own core docs, which describe `wm` itself rather than the adopting project. No `rules/` pages and no memory entries are seeded: a project's rules are the owner's to write, the same principle that removed the global rule layer, and seeding invented policy would contradict it.

  **Sequencing.** The init path calls the builder, so it lands **after** the builder exists. `apps/wm-cli` is therefore in scope for this spec, though not for the first landing.

  - [ ] AC-12: `wm init` on an empty project creates the four core starter pages as conformant record bodies (schema_version, state, canonical questions, answers).
  - [ ] AC-13: Re-running `wm init`, or running it on a project that already has those pages, changes nothing - no overwrite, no duplicate.
  - [ ] AC-14: The seeded pages contain project-placeholder scaffolding, not this repository's own core content.

  ## Order of record (2026-10-16)

  1. **This spec's build, first landing** - the create-path builder, record bodies, `extra_frontmatter` on create, envelope pass-through, hand-rolled YAML removed. In flight in `apps/wm-core`; no other writer enters that crate until it reports.
  2. **Init seeding, second landing** - AC-12 to AC-14, the four core starter pages, content in @wiki/reference:wm-init-starter-pages. Blocked on (1) because the init path calls the builder.
  3. **`cwr-01` onward** - @wiki/specs:configurable-wm-root starts only after (1) reports, so `apps/wm-core` has one writer at a time. `cwr-05` is where `wm init --wm-root` lands, which is the capability that began this thread.

  The seeding scope was added on 2026-10-16 after a terminology collision: "create path" was read as the page-creation code rather than the path where `.wm` is created. Owner reviewed and kept it - the empty-wiki gap is real, since `wm init` writes no pages while the shim it writes points at four core pages that do not exist.
questions:
  - id: kind
    type: choice
    instructions: What kind of spec is this?
    options:
    - feature
    - system
    - doc
    - migration
    - experiment
  - id: scope
    type: choice
    instructions: How wide is the scope of this spec?
    options:
    - local
    - component
    - system
    - project-wide
  - id: status_class
    type: choice
    instructions: What lifecycle class is this spec in?
    options:
    - draft
    - reviewed
    - approved
    - superseded
  - id: needs_tasks
    type: noul
    instructions: This spec requires one or more task pages.
answers: {}
