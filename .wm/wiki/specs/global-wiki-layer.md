---
title: Global wiki layer — two source roots composed into one engine
type: spec
id: "wiki:specs:global-wiki-layer"
status: superseded
tags: [spec, global, multi-root, architecture, search]
relates_to:
  - {type: supersedes, target: wiki:specs:global-rules-cascade}
acceptance_criteria:
  - checked: false
    text: 'AC-1: With a fixture project root and fixture global root, a global `concept` page is found by `wm_search`, appears in the graph, and a `relates_to` edge from a project page to it resolves in both directions.'
  - checked: false
    text: 'AC-2: Same `(type, slug)` in both roots → project effective, global shadowed; the shadow is surfaced in `wm_initial` and in audit output.'
  - checked: false
    text: 'AC-3: Create with `scope=global` lands in `$HOME/.wm/wiki` and is immediately searchable in the same process (write-path refresh); the response reports the actual destination path, with no echo defect of the `layer` kind.'
  - checked: false
    text: 'AC-4: `scope=global` with `task`/`spec`/`core` → validation error, zero bytes written; a hand-placed `task` file in the global root is excluded from the index with a lint warning. Both tested.'
  - checked: false
    text: 'AC-5: `promote` uses move semantics; the graph node and edge set is identical before and after (identity stability); destination collision → error plus diff, no bytes written; a shadow warning appears when the identity exists in another project root.'
  - checked: false
    text: 'AC-6: An external modification in the global root triggers a debounced refresh (integration test), or the documented warning is emitted when watch registration fails.'
  - checked: false
    text: 'AC-7: With both `HOME` and `USERPROFILE` unset, resolution returns a hard error and never a `.`-relative path (regression test against the `memory.rs:194` shape).'
  - checked: false
    text: 'AC-8: Exactly one `EngineStateMediator` in the two-root path; the `Cargo.toml` diff adds zero dependencies; no `#[allow(dead_code)]`; `cargo build`/`clippy` warning-free; no external services.'
  - checked: false
    text: 'AC-9: The `Global` memory variant and the memory `Promote` action are gone (per @wiki/specs/remove-global-memory-layer); global memory pages remain reachable through page list and search.'
  - checked: false
    text: 'AC-10: `WIKI-MEM.md`, the six `embed_files/` shims, their repo-root twins and the `wm-init` skill describe the two-root model identically — byte-parity enforced in lint/CI.'
  - checked: false
    text: 'AC-11: `wm-cli global audit` run against the five real projects reproduces the evidence table — at least two same-identity-different-SHA pairs and a non-empty name-cluster report.'
  - checked: false
    text: 'AC-12: `ignore_global_rules`-equivalent opt-out (`ignore_global_wiki`) yields project-only composition, verified by test.'
---

schema_version: 1
state: |-
  ## Overview

  Today every wm project owns exactly one wiki root: `<project>/.wm/wiki/`. Knowledge that recurs across projects is therefore re-authored per project, and it has measurably diverged. This spec makes the wiki layer global: the engine composes **two source roots** — the project wiki and the user-global wiki at `$HOME/.wm/wiki/` — into a single graph, search index and section index.

  Scope is **provenance, not page data**. A page's identity stays `(type, slug)` regardless of which root supplied it; the project copy is effective and the global copy is shadowed and reported.

  This spec supersedes @wiki/specs/global-rules-cascade (mechanism withdrawn; its precedence rule, audit command, move-promote semantics and curation plan carry forward) and is independent of, but complementary to, @wiki/specs/remove-global-memory-layer

  ## Evidence

  Measured 2026-10-15 across five wm projects under `/Users/nhkhanh/projects`. Rule pages: `wiki-mem` 18 · `eightcap-new-portal` 16 · `axon-ai-course` 6 · `fish-deckbuilding` 6 · `idle-incremental-game` 0.

  **One concept, four names — and divergent content under matching names:**

  | Concept | axon-ai-course | eightcap-new-portal | fish-deckbuilding | wiki-mem |
  |---|---|---|---|---|
  | no comments in code | `no-comments-naming-only` (2361 B) | `no-code-comments` (2051 B) | `no-comments-in-code` (498 B) | `no-comments-in-code` (2149 B) |
  | spec-first | `spec-first` (2437 B) | `spec-first-workflow` (1983 B) | `spec-before-action` (401 B) + `spec-driven-development` (476 B) | `findings-first-task-spec` (2577 B) |
  | TDD | `tdd-red-green-refactor` (1996 B) | — | `tdd` (1537 B) | `tdd-red-green-refactor` (3361 B) |

  `no-comments-in-code` is 498 B in one project and 2149 B in another — different SHA-256. `no-magic-values` is 508 B vs 3190 B. The same name denotes different semantics: false confidence, not merely duplication. Coverage is uneven too — `no-else` exists in one project only.

  A de-facto cross-project rulebook already exists outside wm: `/Users/nhkhanh/projects/AGENTS.md` (5,907 B). wm does not read it, and no project's `.wm/config.json` references an outside root.

  **Cost of the change (verified):** both derived-state builders already take a directory, so the change is a signature plus a merge step rather than a redesign — `build_graph_from_wiki` (`apps/wm-core/src/graph/mod.rs:51`), `build_sections_from_wiki` (`freshness_service.rs:20`), driven by `let wiki_dir = root.join(WM_DIR).join(WIKI_DIR);` (`freshness_service.rs:14`) and `engine.rebuild_graph(&wiki_dir);` (`:19`). The containment gate `is_wiki_page(wiki_dir, wiki_dir_real, path)` (`main_engine_factory.rs:97`) and the watcher path (`main_engine_factory.rs:263`) are the other two touch points.

  ## Decision

  - **D1 — Two source roots, one engine, one index.** The engine composes `<project>/.wm/wiki/` and `$HOME/.wm/wiki/` into a single graph, BM25 index and section index. Neither root gets its own engine or index.
  - **D2 — Identity is `(type, slug)`; scope is provenance.** No frontmatter `scope` field, no id namespacing. A page from the global root is structurally identical to a project page. Consequences: zero migration, existing graphs and edges keep resolving, and promotion is pure relocation.
  - **D3 — The project shadows the global by identity, and shadowing is reported.** When both roots supply the same identity, the project copy is effective and the global copy is shadowed. Shadowing surfaces in `wm_initial` and in the audit command — never silent.
  - **D4 — A closed shareable set.** Shareable: `rule`, `pattern`, `concept`, `howto`, `reference`, `decision`, `memory`. Project-local only: `task`, `spec`, `core`, `index.md`. Enforced twice — at the write path (validation error) and at index merge (excluded plus a lint warning), because `$HOME/.wm/wiki/` is directly editable by hand. Hand-placed non-shareable pages must never silently bind.
  - **D5 — One file, one truth.** Page writes take an optional `scope` (default `project`). Writing a page whose effective copy is global writes the global file. There is no second copy, so there is no drift by construction.
  - **D6 — `promote` and `demote` move.** Move semantics: write to the destination root, delete the origin, refresh derived state. Node identity is unchanged, so every edge survives. A destination collision with different content fails with both paths and a diff, writing nothing, unless `force` or `rename` is given. `promote` additionally warns when the identity is shadowed in other projects.
  - **D7 — Cross-project visibility comes from an advisory registry.** `$HOME/.wm/projects.json`, appended by `wm init`/`wm setup` (deduped by path, tolerant of corrupt or missing entries), is read by the audit command and by `promote`'s shadow warning. It is advisory state: never read on the write-critical path and never an error source. A JSON file, not a database.
  - **D8 — The global root is watched.** The same debouncer covers it. If watch registration fails (network home, platform limits), degrade with a visible warning — "global root not watched; changes by other processes visible after restart" — never silently stale.
  - **D9 — Home resolution errors; it never degrades to the project.** With both `HOME` and `USERPROFILE` unset, resolution returns a hard error. The existing fallback (`memory.rs:194`) must not be carried forward: under a merge, resolving global to the project root would index the project twice and shadow everything.
  - **D10 — `session` is not a scope.** Scope applies to persisted roots only. Session memory (`engine_state_mediator.rs:41`) stays strictly process-local.
  - **D11 — No write propagation between processes.** No IPC, no locks. Markdown plus watcher debounce is the entire consistency protocol; last-writer-wins at the filesystem level.

  ### Rejected alternatives

  - **Shadow-sync with provenance** (`synced_from: global@<hash>` plus a divergence linter). Rejected: it institutionalises the measured disease — copies that drift, detected only by additional machinery. Its lint is a compensating layer for an index limitation that multi-root removes outright. Preserving the letter of "single root" by hiding a second logical store behind copies and hashes violates the spirit.
  - **Read-time overlay only** (the superseded `global-rules-cascade` mechanism). Valid for rules, which are read in full at session start. Invalid for concepts, patterns, howtos and decisions, which are **searched** — an overlay would leave them invisible to `wm_search`, the graph and `wm_initial`.
  - **Symlinking `$HOME/.wm/wiki` into each project.** The realpath containment check in `is_wiki_page` (`main_engine_factory.rs:97`) fails for targets outside the root; duplication without merge gives no shadow semantics; writes through a link have ambiguous scope; and Windows symlink privileges are a problem — the `USERPROFILE` fallback proves Windows is in scope.
  - **Namespacing ids as `wiki:global:*`.** Would churn every `relates_to`/`extends`/`supersedes` edge on every promotion, and require a migration for all existing refs.
  - **A `scope:` frontmatter field on pages.** Encodes location into page data, contradicting D2; the memory layer's `Global` enum variant was exactly this category error.
  - **An external database or a service.** Forbidden by the ARCHITECTURE non-negotiables.

  ## Proposed ARCHITECTURE amendment

  The stated non-negotiable is *single engine per **process***; the single-root assumption is an implementation fact (`project_root: RwLock<PathBuf>`, `engine_state_mediator.rs:35`), not a stated invariant. This spec proposes recording the composed-root reading explicitly:

  > One engine per process. The engine composes **exactly two source roots** — the project wiki (`<project>/.wm/wiki/`) and the user-global wiki (`$HOME/.wm/wiki/`) — into a single graph, search index, and section index. No root gets its own engine or index; both live under one `EngineStateMediator`. Page identity is `(type, slug)`; when both roots supply the same identity, the project copy is effective and the global copy is shadowed-and-reported.

  The invariant's purpose — coherent derived state within one process, no external services — survives intact: one graph, one BM25 index, one write channel, zero new dependencies, zero new processes.

  ## Contract

  **Composition.** `effective = project ∪ global`, keyed by `(type, slug)`.

  **Precedence.** Project wins by identity; the global copy is shadowed and reported.

  **Shareable set.** `rule`, `pattern`, `concept`, `howto`, `reference`, `decision`, `memory`. Non-shareable: `task`, `spec`, `core`, `index.md`.

  **Write.** `scope: project | global` (default `project`); response reports the actual destination path, never an echo of the request.

  **Moves.** `promote` (to global), `demote` (to project) — move semantics, collision refusal with diff.

  **Visibility.** `wm_initial` reports `pages: P project, G global, S shadowed`.

  **Audit.** `wm-cli global audit` reports: same-identity-different-SHA across scopes; the shadow report; name-similarity clusters; and any non-shareable pages ignored in the global root.

  **Registry.** `$HOME/.wm/projects.json` — advisory, best-effort.

  ## Change Inventory

  **Engine**
  - `build_graph_from_wiki` (`graph/mod.rs:51`) and `build_sections_from_wiki` (`freshness_service.rs:20`) accept multiple roots and merge; the call site at `freshness_service.rs:14-20` composes both.
  - `is_wiki_page` (`main_engine_factory.rs:97`) tests containment against any registered root.
  - Engine state holds two roots; still exactly one `EngineStateMediator`.
  - The write path accepts `scope` and refreshes derived state for the affected identity in the owning root.
  - The watcher registers the global root (`main_engine_factory.rs:263`) with a documented degradation path.

  **CLI / MCP**
  - `scope` on page writes; `promote`/`demote` with collision refusal; `wm-cli global audit`; the registry append in `wm init`/`wm setup`.

  **Docs and embedded assets (RustEmbed'd — change in lockstep with repo-root twins)**
  - `wiki:core:ARCHITECTURE` — adopt the amendment above.
  - `wiki:core:CONVENTIONS` — document the cascade, the shadow rule and the closed shareable set (coordinate with @wiki/specs/remove-global-memory-layer, which edits the adjacent memory guidance).
  - `WIKI-MEM.md` + the six shim twins + repo-root `WIKI-MEM.md` — describe the two-root model.
  - The session-start rules/memory block in `embed_files/skills/wm-init/SKILL.md` **and** `.opencode/skills/wm-init/SKILL.md` — three specs touch this block (`remove-global-memory-layer` removes the global *memory* load; `global-rules-cascade` is superseded; this spec adds the global *wiki* overlay and the shadow counts). **They must land together or the session-start instructions become internally inconsistent.**

  ## Migration

  **None.** Identity is unchanged, so every existing page id, ref and edge keeps resolving: `wiki:rules:...`-style ids are already location-free. Pages added to the global root are new files. The only behaviour change is that knowledge found in the global root becomes visible to search and the graph.

  ## Acceptance Criteria

  - [ ] AC-1: With a fixture project root and fixture global root, a global `concept` page is found by `wm_search`, appears in the graph, and a `relates_to` edge from a project page to it resolves in both directions.
  - [ ] AC-2: Same `(type, slug)` in both roots → project effective, global shadowed; the shadow is surfaced in `wm_initial` and in audit output.
  - [ ] AC-3: Create with `scope=global` lands in `$HOME/.wm/wiki` and is immediately searchable in the same process (write-path refresh); the response reports the actual destination path, with no echo defect of the `layer` kind.
  - [ ] AC-4: `scope=global` with `task`/`spec`/`core` → validation error, zero bytes written; a hand-placed `task` file in the global root is excluded from the index with a lint warning. Both tested.
  - [ ] AC-5: `promote` uses move semantics; the graph node and edge set is identical before and after (identity stability); destination collision → error plus diff, no bytes written; a shadow warning appears when the identity exists in another project root.
  - [ ] AC-6: An external modification in the global root triggers a debounced refresh (integration test), or the documented warning is emitted when watch registration fails.
  - [ ] AC-7: With both `HOME` and `USERPROFILE` unset, resolution returns a hard error and never a `.`-relative path (regression test against the `memory.rs:194` shape).
  - [ ] AC-8: Exactly one `EngineStateMediator` in the two-root path; the `Cargo.toml` diff adds zero dependencies; no `#[allow(dead_code)]`; `cargo build`/`clippy` warning-free; no external services.
  - [ ] AC-9: The `Global` memory variant and the memory `Promote` action are gone (per @wiki/specs/remove-global-memory-layer); global memory pages remain reachable through page list and search.
  - [ ] AC-10: `WIKI-MEM.md`, the six `embed_files/` shims, their repo-root twins and the `wm-init` skill describe the two-root model identically — byte-parity enforced in lint/CI.
  - [ ] AC-11: `wm-cli global audit` run against the five real projects reproduces the evidence table — at least two same-identity-different-SHA pairs and a non-empty name-cluster report.
  - [ ] AC-12: `ignore_global_rules`-equivalent opt-out (`ignore_global_wiki`) yields project-only composition, verified by test.

  ## Verification

  1. Fixture-root tests for composition, precedence, the closed set, write scope and promote/demote collision refusal.
  2. `wm-cli global audit` against the five real projects — both report classes must be non-empty on today's data.
  3. `wm_initial` inspected with and without the opt-out.
  4. `rg` gate: no second engine/index object; no new dependency in the lockfile.
  5. Read-back of every edited wiki page to confirm frontmatter and record body survived (rule `durable-write-or-error`).

  ## Limitations and Risks

  - **Index-merge cost at scale.** Fine at knowledge-base sizes; if a global root grows to thousands of pages, merging two full walks per build may need incremental sharding. Flip signal: a measured build-time regression, not speculation.
  - **Shadow silence.** A project fork quietly defeating a global update is the residual false-confidence risk. Mitigated by three reported surfaces (counts, audit, promote warning).
  - **Single point of failure.** A corrupt or missing `$HOME/.wm/wiki` affects every project's effective wiki. Plain markdown, so it is git-init-able; cross-machine sync is out of scope until demanded.
  - **Concurrent global writes** are last-writer-wins. Acceptable at agent editing speeds.
  - **Windows is in scope** by construction (`USERPROFILE`). `~/.wm` semantics, the symlink-free design and watcher behaviour need CI coverage, or these invariants are aspirational.
  - **The mechanism prevents new divergence; it does not heal existing divergence.** `no-comments-naming-only` and `no-comments-in-code` will not be recognised as one intent by any machinery. Curation is human, and must land with or before the mechanism — otherwise a project with zero rules (`idle-incremental-game`) suddenly becomes bound by content it never had.

  ## Rollback

  Composition is opt-out via config. Global pages are plain markdown in a plain directory, so the global root can be made a git repository later at near-zero cost, giving audit and rollback. Removing the global root entirely degrades to today's single-root behaviour. Page identity never depended on scope, so no data migration is needed in either direction.

  ## Amendment (2026-10-15) — sequencing reversed: empty global root first

  Owner-approved. The earlier text required curation to land with or before the mechanism, so that no project would be silently bound. That intent is satisfied **more safely by the reverse order**: ship the mechanism with an **empty** `$HOME/.wm/wiki/`, which binds nothing, then curate into it.

  **Rationale.** The stated risk — a project with zero rules suddenly bound, such as `idle-incremental-game` — only materialises if the mechanism ships with uncurated content already present in the global root. An empty root cannot bind anything, so the ordering constraint disappears. It also removes the need to write outside the project root before the composition surface exists, which confinement would otherwise block.

  **Consequences for the task set.** `CURATE-01` splits into `curate-01a` (curation map, scope decisions, AGENTS.md port, ordering decision — executable now) and `curate-01b` (populate the global root, shadow decisions, audit verification — blocked on `gwl-01`, `gwl-02`, `gwl-04`, `gwl-05`).

  ## Superseded (2026-10-15) - the global wiki layer is removed

  Owner decision: **remove the global wiki layer and accept manual per-project duplication.** Reason of record: the global rule set is a statement of the owner's preferences across their projects, and deciding which rules go global and which variant is canonical is the owner's call - the curation pass showed the mechanism pushes that judgement somewhere it does not belong.

  The measured need remains real (46 rule pages across four projects, four names for one rule, same-name content differing 4x) and is now accepted as manual duplication rather than solved by composition.

  **Superseded:** multi-root composition, the scope/provenance model as a *mechanism*, shadow reporting, promote/demote, global audit, opt-out. Tasks gwl-01..gwl-05 and curate-01b are cancelled. No ARCHITECTURE amendment is needed - the existing single-root invariant stands unmodified.

  **Survives deliberately:** wiki:specs/configurable-wm-root (relocating one project's state root is independent of cross-project sharing) and wiki:specs/remove-global-memory-layer (deleting the wrong-typed MemoryLayer::Global). wiki:reference/global-rules-curation-map is retained as an advisory inventory only - no mechanism consumes it.
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