---
title: 'wm-cli inherited violations: core-duplicating search mapping + full-corpus clone'
type: task
id: "wiki:tasks:wm-cli-inherited-violations-core-duplicating-search-mapping--full-corpus-clone"
status: todo
priority: medium
tags: [wm-cli, drY, clone, layering, inherited]
acceptance_criteria:
  - text: "The CLI calls wm_core::search::indexed_doc_from_section and its local indexed_fields plus the three local weight constants are deleted"
  - text: "Weights used by core and the former CLI copy are confirmed identical (or the difference is a decision), with a test that pins the field names and weights"
  - text: "The section corpus is no longer cloned in full: Arc::clone or clone-on-write is used, verified by a test or by an explicit review note"
  - text: "cli_test and cli_characterization_test stay green through both changes"
---

Two pre-existing rule violations carried over verbatim from `apps/wm-cli/src/main.rs` during step 2 of the controlled rewrite (`wiki:tasks:controlled-rewrite-of-appswm-clisrcmainrs--one-architecture-decision-per-review`). Both were deliberately preserved to keep step 2 behaviour-frozen; neither is fixed yet. Filing so they are not lost.

1. **The CLI re-implements core search indexing.** `apps/wm-cli/src/shared/services/engine_service.rs:94` defines `fn indexed_fields(section: &SectionDoc) -> Vec<wm_core::search::Field>` which re-authors the field mapping that already exists as `pub fn indexed_doc_from_section(s: &SectionDoc) -> IndexedDoc` at `apps/wm-core/src/search/mod.rs:16`. The CLI layer duplicates logic that belongs to `wm-core` — a compensating/duplicating layer rather than a caller. Fix direction: call `wm_core::search::indexed_doc_from_section` from the CLI and delete `indexed_fields` and its three local weight constants. Verify the weights agree before switching, and re-run the CLI goldens plus `cli_test`.

2. **Full-corpus `Vec` clone on the index rebuild path.** The `sections.clone()` feeding `section_corpus` is a `Vec<SectionDoc>::clone()` — a full-corpus heap copy, which rule `no-dead-code-clone-scanning` marks as never acceptable (the accepted cheap clone is `Arc::clone`; the documented fix is `Arc::make_mut()` / copy-on-write, see `@wiki/patterns/arc-vec-section-corpus`). Verbatim from the original `main.rs`; carried into `shared/services/engine_service.rs`. Fix direction: adopt the `Arc` / clone-on-write pattern already documented for the section corpus.

Both sit on the index-rebuild path exercised by `cli_test` (`test_cli_index_rebuild`) and the corpus is read by search, so they should be fixed with tests, not by inspection.