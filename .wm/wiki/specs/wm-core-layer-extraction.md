---
title: Extract the application layer from wm-core, and enforce the boundary in CI
type: spec
id: "wiki:specs:wm-core-layer-extraction"
status: approved
tags: [spec, approved, architecture, refactor, wm-core, layering]
relates_to:
  - {type: depends_on, target: wiki:reference:wm-core-architecture-inventory}
---

schema_version: 1
state: |-
  ## Overview

  Get the **application layer out of the library**, and make the crate boundary enforceable in CI so it cannot silently erode again.

  `apps/wm-core` is 22,230 lines across 151 files, and `mcp/` is 8,789 of them - 39.5% of the crate. That is not transport glue: the tool handlers build frontmatter, do task and memory CRUD, walk files and infer languages. Meanwhile a *library* carries the `rmcp` dependency to perform one error conversion that nothing calls.

  **The goal is dependency hygiene and enforced layering - not "loosely coupled".** The tool handlers will keep calling `Arc<EngineState>` domain methods; that behavioural coupling stays and is correct. The old draft spec promised "loosely coupled, independently testable" and was judged against that promise, which is part of why it was collapsed. This spec does not make that claim.

  Outcome: one new crate (`wm-tools`), one merge (`parser` into `wm-engine`), roughly 800 lines of dead code deleted first, and three CI gates.

  ## Evidence

  Measured 2026-10-16; full inventory at @wiki/reference:wm-core-architecture-inventory.

  - **`mcp/` is 39.5% of wm-core** (8,789 / 22,230). Genuine glue is small: `transport.rs` 603, `handler.rs` 53, `prelude.rs` 8. The rest is domain-shaped: `tools/task/mod.rs` 994, `tools/code.rs` 954 (re-walks files and infers languages, duplicating `wm-code-intel`), `tools/memory.rs` 481.
  - **The `rmcp` feature in wm-core is dead, not merely heavy.** `impl From<ToolError> for ErrorData` (`error/mod.rs:166-177`) has **zero call sites**: `rg ToolError apps/wm-cli/src` returns nothing, and `wm-cli/src/mcp_server.rs` constructs its own `ErrorData` directly at `:43`. The library therefore pulls `rmcp` and its transitive tree for an unused conversion. Verified 2026-10-16.
  - **The seam already exists and already has two consumers.** `ToolRegistry::dispatch_async` (`mcp/transport.rs:263`) is called from `wm-cli/src/mcp_server.rs:62` (rmcp stdio) and `wm-cli/src/shared/services/engine_proxy.rs:37` (the second transport path). Nothing needs designing; it needs moving.
  - **`error/` cannot ride along.** 36 non-`mcp` files in wm-core reference `ToolError` (page services, parser, version repo, shared traits), so extracting the error type would require giving 36 files new domain error types.
  - **Dead code**: `task_board_service.rs:18-58` has zero call sites while `mcp/tools/task/mod.rs:126-213` reimplements it; `wm-engine/src/engine_state.rs:3` declares a unit-stub `EngineState`; `wm-core/src/vector_db/mod.rs` is one line re-exporting `wm_embed::vector_db::*`.
  - **wm-core is a cyclic mesh, not a stack** (measured 2026-10-16 for D-11). Cross-module `crate::` imports: `page/` to parser x18/engine x11/error x9; `graph/` to engine x11/parser x10/search x5; `engine/` to graph x5/search x4/config x4; `search/` to engine x6; and `mcp/` fans into everything (engine x41/parser x32/page x23). page, graph and engine reference each other through `EngineState`, so a module-per-crate split would need a trait per consumer - the ceremony that got task `e688f0` merged back.

  ## Decision

  **D-1 - A crate must earn its existence. Extraction requires at least one of:**

  - **(a) Mass:** 1,500+ lines with a closed public interface and an independent test story; or
  - **(b) Dependency-extraction value:** removing it takes a heavyweight dependency (rmcp, tree-sitter, ort) out of someone's tree; or
  - **(c) Multi-consumer:** two or more consumers in different binaries or processes.

  A **merge** into an existing crate is exempt from the floor - merging adds no ceremony. This reconciles with the critique that collapsed the previous attempt: `wm-shared` (28 lines) failed all three *and was a new crate*; `mcp/` (8,789) passes all three.

  **D-2 - The seam set.** Exactly one new crate and one merge:

  | Action | Target | Size | Grounds |
  |---|---|---|---|
  | Extract | `mcp/**` to `packages/wm-tools` | 8,789 | (a) and (b) and (c) |
  | Merge | `parser/` (1,558) into `wm-engine` as `wm_engine::parser` | 1,558 | kills the duplicated models by construction; a merge, so exempt from D-1 |
  | Delete | `task_board_service.rs`, wm-engine's `EngineState` stub, the `vector_db` shim, the `rmcp` feature and its dead conversion | ~800 | zero call sites, verified |
  | Keep in wm-core | `graph/` 2,597, `page/` 3,094, `engine/` 1,108, `search/` 981, `version/` 801, `decision/` 506, `shared/` 519, `config/` 457, `skill/` 433, `source_service` 425, `error/`, `page_repo/` 133, `util/` 62 | | sub-threshold, or one consumer, or entangled |

  **D-3 - Explicitly not extracted, however tidy they look.** `graph/` is entangled with `EngineState` and shares `parse_page`/`add_page_edges` between its full and incremental paths, so extracting it would need a trait per consumer - ceremony without decoupling. `page/` is the domain layer the tools call, with exactly one consumer. `page_repo/` (133 lines) is sub-threshold on every axis. `config/`, `skill/`, `shared/`, `util/` are sub-threshold with one consumer.

  **D-4 - Rulings on the superseded draft's 12 packages.** Keep `wm-embed`, `wm-engine`, `wm-search`, `wm-code-intel`, `wm-lsp` (all exist and earn their keep). **Merge** `wm-parser` into `wm-engine` - standalone it would recreate the model duplication it exists to fix. **Drop** `wm-error` (36 domain consumers would all need new error types; that is a shared type, not a library), `wm-status` (~250 lines of enums belonging beside `PageStatus`), `wm-util` (62 lines - the exact failure mode), `wm-page-repo` (133 lines, one impl), `wm-template-engine` (sub-threshold, one consumer), `wm-config` (457 lines, one consumer), `wm-vector-db` (it already *is* `wm-embed/src/vector_db.rs`; the wm-core side is the shim being deleted).

  **D-5 - The tool layer becomes `packages/wm-tools`, not a folder in `wm-cli`.** `wm-cli` already has two consumers of the registry, and an HTTP/WASM server is on the roadmap. Folding 8,800 lines of handlers into the binary would make the next server link the whole clap/init apparatus to reuse one registry, and would make "handlers must not depend on CLI internals" a convention instead of a compile error. Resulting layers: `wm-cli` (binary + transports) to `wm-tools` (tool handlers + `ToolRegistry`) to `wm-core` (domain library) to the six `packages/*` leaves.

  **D-6 - Extraction order.**

  0. **Dedup and dead-code pass, inside wm-core, zero new crates.** Delete the four dead items; consolidate `slugify` (5 copies) to one and the frontmatter splitter (4-5 copies) to one. The YAML frontmatter serializer (3 copies) is **deferred** to the D-10 follow-up - it needs a type change and a page migration, not a move. Extracting dirty code copies the mess and doubles every later diff.
  1. **Merge `parser/` into `wm-engine`.** Collapse `Frontmatter` (~74 fields) into `WikiPageMeta`; delete one page-type mapping. Before step 2 so the tools' `crate::parser::` imports already point at the new home, leaving fewer cross-boundary edges to rewrite. Keep it a non-breaking submodule addition so `wm-code-intel`, which depends on wm-engine, does not move.
  2. **Extract `mcp/**` to `packages/wm-tools`.** `error/` stays behind. wm-tools declares no `rmcp` and no `clap`.
  3. **Rewire `wm-cli`**: `wm_core::ToolRegistry` to `wm_tools::ToolRegistry`. Same change as step 2 or immediately after; any transitional re-export shim is dropped rather than left - shims are how `vector_db/mod.rs` happened.
  4. **Gates green** (D-7).

  **D-7 - Three CI gates, because conventions did not hold last time.**

  - **Gate 1 - crate floor with a waiver file.** For every `packages/*` member, count source lines; a crate not in a checked-in baseline must clear 1,500 lines *or* appear in `.github/workspace-crates.toml` with a `reason` naming which of (a)/(b)/(c) it satisfies. The waiver is the honest mechanism - `wm-constants` is small but genuinely shared, and would get a reason; `wm-shared` at 28 lines had nothing writeable in that field. **The floor has a ceiling twin (D-11):** a residual `apps/wm-core` must stay at or below a checked-in line ceiling (13,000), waiver-required above it, so the extraction cannot silently erode back.
  - **Gate 2 - dependency deny-list per layer.** The six deny-listed dependencies are enforced by a per-layer allow-map (ratified 2026-10-16): `rmcp` and `clap` may be declared only by `apps/wm-cli`; `notify`, `tracing-subscriber`, `tracing-appender` and `rust-embed` may be declared by `apps/wm-cli` or `apps/wm-core` (wm-core's watcher, logging and embed are legitimate); and **no `packages/*` crate may declare any of the six**. This still catches the rmcp-in-library defect the day it is added, and it is what keeps wm-tools transport-free. The literal "wm-cli only" reading was not satisfiable against the current tree without relocating wm-core's watcher/logging/embed dependencies - a separate change.
  - **Gate 3 - singleton-helper grep.** No duplicate implementations among the distinct-named helpers, plus exactly one page-type-to-string mapping, asserted in CI (reworded 2026-10-16): each of `label_slug`, `memory_slug`, `task_page_id_slug` and each frontmatter splitter must appear exactly once. The original "exactly one `pub fn slugify`" wording was wrong - no such symbol exists, and the recorded decision (name distinct behaviours, do not converge them) keeps them under honest distinct names.

  **D-8 - Accepted debt, recorded rather than hidden.** `ToolError` stays in wm-core with its 36 consumers (fixing it needs a domain-error split, a separate change). wm-tools keeps calling `Arc<EngineState>`. The `WIKI_DIR` constants collision in wm-code-intel is a semantic bug, not a packaging one, and gets its own task rather than riding along with a move.

  **D-9 - This supersedes both prior positions.** `wiki:tasks:e688f0` (the collapse) and `wiki:specs:extract-packages-from-wm-core` (the 12-package draft, never executed: none of the packages exist, and its own "~13K lines" is stale against 22,230).

  **D-10 - Step 0's YAML-serializer unification is deferred to a dedicated follow-up (2026-10-16).** Unifying the three YAML frontmatter serializers requires extending the `FrontmatterValue` model to express block sequences, deciding which byte output wins for six shapes that exist in the live wiki, and migrating the affected pages. That is a change in its own right, not a step-0 tidy-up, so it is split out to keep this spec's blast radius to deletions and consolidations. The follow-up is owned by task @wiki/tasks/unify-yaml-frontmatter-serializers and spec @wiki/specs/yaml-frontmatter-serializer-unification, which carry the type change, the byte-output decision, and the migration. AC-6 is scoped accordingly (the serializer clause moves to the follow-up).

  **D-11 - wm-core is not dissolved into multiple packages; the D-2 seam set stands (2026-10-16).** An independent review evaluated dissolving wm-core entirely (module-per-crate) against D-1 and the measured mesh in Evidence: only `mcp/` clears all three axes; `page/` clears mass alone; `graph/`, `engine/`, `search/`, `version/`, `error/`, `page_repo/`, `util/` fail on closed-interface and single-consumer. Dissolution would produce about nine waiver/ceremony crates and take the workspace from 8 members to 17-19, and the in-repo precedent is task `e688f0`, which tried sub-threshold crates and merged them back. Two hardenings from that review are folded in: (1) Gate 1 gains a wm-core line **ceiling** (D-7), so a residual core cannot silently regrow; (2) the compile-time-parallelism concern is captured as a measurement task @wiki/tasks/measure-wm-core-build-timings, not a structural bet - the `mcp/` move already removes 39.5% and the heavy deps (ort, tree-sitter) are already separate crates. Revisit module-level crates only if `EngineState` is decomposed into per-domain state handles, an out-of-workspace library consumer appears, or `--timings` shows wm-core dominating build time after the extraction.

  ## Duplication disposition

  Mechanical duplicates die in step 0, model duplicates die by the step-1 merge, transport and dependency duplicates die by the step-2 move, and CLI-side duplicates stay out of scope.

  | Finding | Resolved by | Why there |
  |---|---|---|
  | rmcp in the library | Step 0 deletion | The conversion is dead today; waiting would tie a trivial deletion to a risky move |
  | dead task-board twin | Step 0 deletion | It lives at the wm-core lib root, *not* under `mcp/`, so the move never touches it |
  | `EngineState` stub | Step 0 deletion | Pure deletion in wm-engine |
  | `slugify` x5, frontmatter splitter | Step 0 consolidation | Four of five `slugify` copies are inside `mcp/`; fixing late would pollute wm-tools' first commit |
  | YAML frontmatter serializer x3 | Deferred to D-10 follow-up | Needs a `FrontmatterValue` type change + a byte-output decision + a page migration; out of step-0 scope |
  | page-type mapping, frontmatter splitter, `Frontmatter` vs `WikiPageMeta` | Step 1 merge | Uniting parser with the models makes the second copy unrepresentable, not merely deleted |
  | duplicated engine test suites | Step 1 | Both crates' boundary is already being touched; dedupe in the same change |
  | `tools/code.rs` re-walking files | Step 2 AC | It is tools-layer code moving into wm-tools; it must delegate to wm-code-intel rather than arrive with its own traversal |
  | constants collision, CLI duplicates (#7, #8, #14, #15, #17) | Separate tasks | A semantic bug and out-of-scope duplication should not widen this blast radius |

  ## Acceptance Criteria

  - [ ] AC-1: `cargo tree -p wm-core -e normal` contains no `rmcp`, and `rg rmcp apps/wm-core` is empty.
  - [ ] AC-2: `apps/wm-core/src/mcp/` no longer exists; wm-core source is 12,000 lines or fewer, measured.
  - [ ] AC-3: Exactly one new workspace member (`wm-tools`); Gates 1, 2 and 3 are merged and green, and Gate 1 asserts the wm-core line ceiling (D-7/D-11).
  - [ ] AC-4: `cargo build`, `test` and `clippy --workspace` are green; the test count is no lower than pre-extraction minus only the deleted dead suites (named: the wm-engine/wm-core engine test twins).
  - [ ] AC-5: One integration test dispatches `wm_search.query` through **both** transports - the stdio path (`mcp_server.rs`) and the second path (`engine_proxy.rs`) - and asserts identical results. This is the proof the extraction served its stated purpose.
  - [ ] AC-6: No duplicate implementations among the distinct-named helpers, and exactly one page-type-to-string mapping workspace-wide, asserted by grep counts in CI (Gate 3; reworded 2026-10-16 - the literal "one `slugify`" was wrong). The YAML frontmatter serializer clause is deferred to D-10 and asserted by the follow-up spec.
  - [ ] AC-7: `tools/code.rs` delegates file walking and language inference to wm-code-intel; wm-tools adds no new `walkdir`/`ignore` traversal.
  - [ ] AC-8: The spec records as accepted debt with follow-up task links: `ToolError` remaining in wm-core (D-8); the `EngineState` coupling in wm-tools (D-8); the constants/read-policy collision (@wiki/tasks/wmcode-read-policy-should-follow-gitignore-not-a-hardcoded-hidden-file-ban); the deferred YAML serializer unification (D-10, @wiki/specs/yaml-frontmatter-serializer-unification); the duplicated `tests/helpers/setup.rs` (see the EXTRACT-03 notes); and the three pre-existing clippy warnings (@wiki/tasks/fix-pre-existing-clippy-warnings).
  - [ ] AC-9: `cargo build --timings` is captured before and after the `mcp/` extraction and recorded on @wiki/tasks/measure-wm-core-build-timings; the result is the trigger for any future module-level split (D-11).

  ## Risks

  - **Behavioural coupling survives.** This is repackaging for dependency hygiene, not decoupling. Reviewers expecting loose coupling will judge it a failure while it succeeds mechanically - hence the Overview states the goal precisely.
  - **Import-rewrite blast radius:** roughly 30 distinct cross-boundary import paths. Land it as one mechanical change; if it must split, split *file moves first, import rewrite second* - never half the tools moved.
  - **The step-1 model collapse touches wm-code-intel** through wm-engine. Keep parser a non-breaking submodule addition; rerun code-intel's tests if `WikiPageMeta` fields change.
  - **What would change the plan:** the HTTP-server roadmap being formally cancelled (then fold tools into wm-cli and skip `wm-tools`); discovery that wm-core is consumed as a library by a process not in this workspace (then the shared-types question reopens); or measurement showing the parser merge forces more than 200 lines of change in wm-code-intel (then a standalone `wm-parser` after all).

  ## Rollback

  Steps are ordered so each is independently revertible: step 0 is deletions and consolidations, step 1 is a submodule addition, step 2 is a directory move. No data or on-disk format changes; nothing here touches the wiki. Reverting step 2 restores the library without touching steps 0 or 1.

  ## Amendment 2026-10-16 - step 0 item 7 is blocked on a type change, not a code move

  Step 0's serializer consolidation was attempted and **stopped**, correctly, at the guard that says not to change the bytes written for shapes that exist in the live wiki. The blocker is structural:

  - **`frontmatter_to_yaml` is a live writer**, reached through `page/services/timer_recovery_service.rs:59-64` (task pages with a `time_started` older than 24 hours); five such pages exist today. It is not a dormant second copy.
  - **Six shapes would change bytes**, each with a live example: forced double-quoting vs `yaml_scalar`; multi-line literal blocks (`|-`) emitted instead as a newline inside `"..."`, which is malformed YAML; block sequences vs flow maps for `acceptance_criteria` and `relates_to`; flow vs block `tags`; unknown keys such as `spec:`; and
  - **`FrontmatterValue` cannot express block sequences at all** - it is `Scalar | Id | Int | List(Vec<String>) | Nested(Vec<(&str, FrontmatterValue)>)`. Shapes 2 and 3 therefore cannot be produced by a unified builder without **extending the value model**.

  So "one implementation" here requires a type change plus a decision about which byte output wins, followed by a migration for the affected pages. That is a change in its own right, not a step-0 tidy-up. Item 7 is **deferred to D-10**, which assigns it a dedicated follow-up task + spec, and the deferral is recorded here rather than left as an unexplained gap. Steps 1-4 of D-6 are unaffected.
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