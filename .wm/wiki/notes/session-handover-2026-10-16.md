---
title: Session handover 2026-10-15/16
type: note
id: "wiki:notes:session-handover-2026-10-16"
status: reviewed
tags: [note, handover, wm-core, extraction]
relates_to:
  - {type: references, target: wiki:specs:wm-core-layer-extraction}
---

schema_version: 1
state: |-
  Written at the end of the 2026-10-15/16 session, on the owner's instruction that everything remaining be picked up in a fresh session. Read this before touching `apps/wm-core`.

  ## Committed this session

  - `c8e0cbe` `feat(wm-core): shared conformant create builder` - verified: `mcp_test` 99 passed / 0 failed, clippy clean for every changed file, the 8 adapted tests audited and each relocation preserves its assertion's intent.
  - `e5b2c71` `docs(wiki): capture wm-core layering spec and inventory`.
  - `d3c8b9e`, `8ce1646`, `d019dd4` - the global-wiki reversal, outstanding wiki content, and the guard-ordering pattern.
  - Three earlier commits on this branch are **not** mine (HEAD advanced twice from a parallel session during the work).

  ## In the tree, uncommitted

  Step 0 of the extraction has **partly landed but is not committed**:

  - Deleted: `apps/wm-core/src/task_board_service.rs`, `packages/wm-engine/src/engine_state.rs`, `apps/wm-core/src/vector_db/mod.rs`. All three verified zero call sites first - except `vector_db`, which had two live consumers and was repointed rather than deleted blindly.
  - `rmcp` removed from the library: `cargo tree -p wm-core -e normal` now reports **0** rmcp (verified). `wm-cli` keeps its own.
  - Frontmatter splitter consolidated to one byte-preserving implementation with 3 pinning tests.
  - `Cargo.lock` updated as a consequence.

  **Step 0 is now complete except item 7, and nothing is in flight.** The lane `ses_ebd41d867ffe41blk6gQ0R8q5b` finished: item 5 named the three slug behaviours (`label_slug`, `memory_entry_id_slug`, `task_page_id_slug`), collapsed only the byte-identical task copies, and added one pinning test per behaviour; item 7 stopped on the byte-change guard (see the amendment on @wiki/specs:wm-core-layer-extraction - `FrontmatterValue` cannot express block sequences, so unifying needs a type change plus a migration decision). Verified: `cargo check -p wm-core` ok, `cargo test -p wm-core --lib` 247 passed, `mcp_test` 99 passed, clippy with no new warnings.

  **First task in the next session: commit step 0** as one change (deletions, the rmcp removal, the splitter consolidation, the slug naming, `Cargo.lock`), then file nothing else against it.

  `.wm/state/` (3 binary files) is intentionally never committed.

  ## Decisions already made - do not re-litigate

  - **`slugify`:** keep the distinct behaviours, name them honestly, no renames. See @wiki/patterns:name-distinct-behaviours-do-not-converge-them.
  - **YAML serializers:** unify the two field-builders with pinned round-trip tests; leave `graph/export.rs` alone - it is a merge that preserves unknown keys, a different job.
  - **Extraction shape:** one crate out (`mcp/**` to `packages/wm-tools`), one merge (`parser` into `wm-engine`), mass-gated at 1,500 lines with a waiver file, plus three CI gates. Full plan: @wiki/specs:wm-core-layer-extraction.

  ## Next, in order

  1. **Reconcile and commit step 0**, including the running lane's items.
  2. **Extraction steps 1-4** from @wiki/specs:wm-core-layer-extraction: parser merge, `wm-tools` extraction, wm-cli rewire, CI gates. Note the spec supersedes both @wiki/tasks:e688f0 (annotated as reversed) and @wiki/specs:extract-packages-from-wm-core.
  3. **`wm_doc.create` gap** - a parallel session filed it writing flat prose for record-bearing types. Fold it into the shared builder or declare it out of scope; recorded in @wiki/specs:create-path-record-conformance.
  4. **The 4 AC-less tasks** still fail `wm_validate.check`. Enforcement is `depends_on` the extra_frontmatter defect, which has now landed, so it is unblocked.
  5. **`cwr-01` to `cwr-05`** (relocatable state root, `wm init --wm-root`) - untouched this session, spec approved, independent of the extraction.
  6. **Unresolved wiki refs: 2, both intentional syntax examples** in `core/CONVENTIONS`. The former third was `file-size-budget`, resolved as an approval-gated recommendation rather than a missing rule.

  ## Process notes

  - A parallel session is active in this repo. Check `git log` before assuming a change is yours.
  - **Verify targeted while iterating, in full only at the end.** Owner instruction; in @wiki/memory.
  - Two lanes this session stopped on behavioural divergences instead of choosing - that is the intended behaviour, not a blockage to route around.
questions:
  - id: kind
    type: choice
    instructions: What kind of note is this?
    options:
    - note
    - index
    - scratch
    - changelog
answers: {}