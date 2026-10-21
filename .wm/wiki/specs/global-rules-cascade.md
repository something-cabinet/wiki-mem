---
title: Global rules cascade — cross-project rule sharing
type: spec
id: "wiki:specs:global-rules-cascade"
status: superseded
tags: [spec, rules, cross-project, cascade, cli]
relates_to:
  - {type: relates_to, target: wiki:specs:remove-global-memory-layer}
acceptance_criteria:
  - checked: false
    text: 'AC-1: Session-start rules load returns `global ∪ project` with project winning on slug collision; unit test with fixture dirs covers the shadow case.'
  - checked: false
    text: 'AC-2: `rule promote` moves (copy + delete) atomically; a collision with different content errors, naming both paths and a diff, and writes zero bytes; `force` and `rename` paths are tested.'
  - checked: false
    text: 'AC-3: Editing a global rule returns footprint counts (effective in N projects, shadowed in M).'
  - checked: false
    text: 'AC-4: `wm-cli rules audit` reproduces the measured findings as an integration fixture — same-slug-different-SHA ≥ 2 and a non-empty name-cluster report.'
  - checked: false
    text: 'AC-5: No second engine or index object anywhere; an `rg` gate confirms no new `EngineStateMediator`-like state holding `~/.wm` paths.'
  - checked: false
    text: 'AC-6: `ignore_global_rules: true` yields project-only rules, verified by test.'
  - checked: false
    text: 'AC-7: Shim parity — `embed_files/` and repo-root twins are byte-compared in CI or lint (this guard becomes load-bearing here).'
  - checked: false
    text: 'AC-8: `wm_initial` prints global / project / effective / shadowed counts.'
  - checked: false
    text: 'AC-9: `resolve_global_wiki_dir` errors, and does not fall back to the project, when both `HOME` and `USERPROFILE` are unset.'
  - checked: false
    text: 'AC-10: `cargo build` and `cargo clippy` are warning-free; no new `#[allow(dead_code)]`.'
---

schema_version: 1
state: |-
  ## Overview

  The repo owner's five wm projects each carry their own `.wm/wiki/rules/` pages, and the same concepts have been re-authored — and re-named — per project. This spec introduces a **global rules cascade**: global rules live once at `~/.wm/wiki/rules/`, and each project's effective ruleset is `global ∪ project` with the project shadowing by slug.

  This is deliberately **not** the `global` memory layer (see @wiki/specs/remove-global-memory-layer, a separate spec). The memory-global skeleton is `type: memory`; rules are `type: rule`. Completing the memory store would deliver none of the benefit described here.

  ## Evidence

  Measured 2026-10-15 across the five wm projects under `/Users/nhkhanh/projects`.

  **Rule-page counts:** `wiki-mem` 18 · `eightcap-new-portal` 16 · `axon-ai-course` 6 · `fish-deckbuilding` 6 · `idle-incremental-game` 0.

  **Naming divergence — one concept, four names:**

  | Concept | axon-ai-course | eightcap-new-portal | fish-deckbuilding | wiki-mem |
  |---|---|---|---|---|
  | no comments in code | `no-comments-naming-only` (2361 B) | `no-code-comments` (2051 B) | `no-comments-in-code` (498 B) | `no-comments-in-code` (2149 B) |
  | spec-first | `spec-first` (2437 B) | `spec-first-workflow` (1983 B) | `spec-before-action` (401 B) + `spec-driven-development` (476 B) | `findings-first-task-spec` (2577 B) |
  | TDD | `tdd-red-green-refactor` (1996 B) | — | `tdd` (1537 B) | `tdd-red-green-refactor` (3361 B) |

  **Content divergence where names match:** `no-comments-in-code` is 498 B (fish-deckbuilding) vs 2149 B (wiki-mem) — different SHA-256. `no-magic-values` is 508 B vs 3190 B — different SHA. The same rule name therefore denotes different semantics in different repositories: false confidence, not merely duplication.

  **Coverage gaps:** `no-else` exists only in wiki-mem; `no-magic-values` in 2 of 4 projects with rules.

  **A de-facto rulebook already exists outside wm:** `/Users/nhkhanh/projects/AGENTS.md` (5,907 B) carries the owner's language and implementation standards, delegation standards and approval gates. wm does not read it. All five `.wm/config.json` files are identical boilerplate (`source_dirs: ["docs/","specs/"]`, which governs *source ingestion*); none references an outside rules root.

  ## Decision

  - **D1 — Read-time cascade, not a store or an index.** The effective ruleset is computed as `global ∪ project` at the rule-loading call site. There is no second engine, no second index, and no change to `wiki:core:ARCHITECTURE`'s "single engine per process". Rules are binding because they are read in full at session start; they do not need to be searchable to be binding.
  - **D2 — Global rules live once, at `~/.wm/wiki/rules/`.** Reuses `WM_DIR`/`WIKI_DIR` so the existing global-directory resolution generalises to `resolve_global_wiki_dir(subdir)`.
  - **D3 — Identity is the filename slug.** A rule is addressed by slug at its canonical location. No content merging of binding prose, ever.
  - **D4 — Project shadows global, wholesale, by slug.** If both exist, the project copy wins entirely. Shadowing is reported, not silent.
  - **D5 — `rule promote` moves, it does not copy.** Copy to global then delete the project copy in the same operation, leaving exactly one copy. Drift becomes structurally impossible. A project that later needs to diverge does so by deliberately shadowing.
  - **D6 — Collisions refuse, they do not overwrite.** Promoting a slug whose global counterpart has different content fails with both paths and a diff, and writes nothing, unless `force` or `rename` is given. Replaces the silent-overwrite behaviour of the memory `promote` path.
  - **D7 — Global writes report their footprint.** Because a global rule binds every project, the write response states how many projects it is effective in and how many shadow it.
  - **D8 — Home resolution must not degrade to the project.** When neither `HOME` nor `USERPROFILE` is set, resolution **errors** ("cannot resolve user home"). The memory path's fallback to `"."` is the worst possible behaviour for a cascade: it would create global-looking writes inside a project root.
  - **D9 — Opt-out exists.** `config.json` gains `ignore_global_rules` so a cascade cannot bind a project that did not consent.
  - **D10 — Curation is human, tooling only reports.** Identity resolution across differently-named rules cannot be automated safely; `wm-cli rules audit` reports the two failure modes and the owner decides.

  ### Rejected alternatives

  - **Complete the memory-global layer instead.** Wrong entity type: rules are `PageType::Rule`, record-bearing, loaded at init, with their own lifecycle. Hanging rules off `wm_memory` is accidental coupling with no call site to justify it — and building it to solve a rules problem is precisely the compensating layer `no-compensating-layers` forbids.
  - **One-shot copy promotion** (the existing `promote` semantics). This is the mechanism that produced the measured drift: two independent copies of one name silently diverging. Rejected on the evidence.
  - **Content merging on collision.** Merging binding rule prose produces an unpredictable third artefact neither author wrote.
  - **Machine-side synonym detection / fuzzy auto-merge.** Would silently change which rule is binding in a project, and masks a curation problem rather than fixing it.
  - **Shadow-copy-with-provenance** (`synced_from: global@<hash>` plus divergence lint). Real complexity budget; deferred. This is the upgrade path the moment global rules must be searchable or per-project divergence must be tracked.
  - **Indexing global rules into the graph/BM25.** Requires a multi-root index, breaching "single engine per process", for pages that are read in full every session.

  ## Contract

  **Precedence.** `effective = global ∪ (project \ shadowed)`; project wins by slug; collisions reported.

  **Resolution.** `resolve_global_wiki_dir(subdir)` resolves `$HOME/.wm/wiki/<subdir>` via `WM_DIR`/`WIKI_DIR`; `HOME` → `USERPROFILE` → **error** (D8).

  **Surface.** A rules-oriented verb set — `promote` (move-to-global), `demote` (move-to-project), and a footprint-aware global write path. Not grafted onto `wm_memory`; the memory `promote` action is removed by the sibling spec.

  **Visibility.** `wm_initial` reports `rules: N global, M project, K effective (S shadowed)` so binding rules are never invisible to the agent they bind.

  **Opt-out.** `config.json: {"ignore_global_rules": true}` → project-only rules.

  ## Curation

  `wm-cli rules audit` scans `~/.wm/wiki/rules/` and every discovered project's `.wm/wiki/rules/`, and reports exactly the two measured failure modes:

  1. **Same-slug, different-SHA** — false confidence: one name, different meaning.
  2. **Name-similarity clusters** — false diversity: four names, one concept. The Evidence table above is this report's first expected output.

  One-time curation for the five measured projects is owner-led: port the canonical set (starting from `/Users/nhkhanh/projects/AGENTS.md` as source material) into `~/.wm/wiki/rules/`, collapse the name clusters, then shadow only where a project genuinely diverges. wm must **not** magic-path-read a per-user file such as `AGENTS.md`; the owner ports the content.

  **Curation must land by the time the cascade ships**, so day-one global content is already owner-blessed. Without it, a project such as `idle-incremental-game` (0 rules today) would suddenly be bound by rules it never had.

  ## Change Inventory

  **Engine**
  - Generalise the global-directory resolver to `resolve_global_wiki_dir(subdir)` with the D8 error behaviour (shared with the sibling spec's refactor).
  - Add the read-time cascade and shadow resolution at the rule-loading call site. No new engine state, no second root.

  **CLI / MCP**
  - `rules` verbs: `promote`, `demote`, footprint-aware global write.
  - `wm-cli rules audit` per the Curation section.

  **Config**
  - `ignore_global_rules` key.

  **Embedded assets (RustEmbed'd — change in lockstep with repo-root twins)**
  - The session-start rules step in `apps/wm-core/src/embed_files/skills/wm-init/SKILL.md` **and** `.opencode/skills/wm-init/SKILL.md` — where the sibling spec removes the global *memory* load, this spec adds the global *rules* overlay load and the cascade counts. Both specs touch this block: **they must land together or the session-start instructions become internally inconsistent.**
  - `embed_files/shims/WIKI-MEM.md` + the six shim twins + repo-root `WIKI-MEM.md` — describe the cascade.
  - `wiki:core:CONVENTIONS` — document the cascade and the shadow rule alongside the sibling spec's memory edit.
  - `wiki:core:ARCHITECTURE` — record that the cascade is a read-time overlay, not a second root, so the "single engine per process" invariant is explicitly reaffirmed rather than quietly bent.

  ## Acceptance Criteria

  - [ ] AC-1: Session-start rules load returns `global ∪ project` with project winning on slug collision; unit test with fixture dirs covers the shadow case.
  - [ ] AC-2: `rule promote` moves (copy + delete) atomically; a collision with different content errors, naming both paths and a diff, and writes zero bytes; `force` and `rename` paths are tested.
  - [ ] AC-3: Editing a global rule returns footprint counts (effective in N projects, shadowed in M).
  - [ ] AC-4: `wm-cli rules audit` reproduces the measured findings as an integration fixture — same-slug-different-SHA ≥ 2 and a non-empty name-cluster report.
  - [ ] AC-5: No second engine or index object anywhere; an `rg` gate confirms no new `EngineStateMediator`-like state holding `~/.wm` paths.
  - [ ] AC-6: `ignore_global_rules: true` yields project-only rules, verified by test.
  - [ ] AC-7: Shim parity — `embed_files/` and repo-root twins are byte-compared in CI or lint (this guard becomes load-bearing here).
  - [ ] AC-8: `wm_initial` prints global / project / effective / shadowed counts.
  - [ ] AC-9: `resolve_global_wiki_dir` errors, and does not fall back to the project, when both `HOME` and `USERPROFILE` are unset.
  - [ ] AC-10: `cargo build` and `cargo clippy` are warning-free; no new `#[allow(dead_code)]`.

  ## Verification

  1. Fixture-based tests for the cascade, shadowing, promote/demote collision refusal, and opt-out.
  2. `wm-cli rules audit` run against the five real projects — same-slug-different-SHA and name-cluster reports must both be non-empty on today's data.
  3. `wm_initial` output inspected in a project with globals, with and without `ignore_global_rules`.
  4. `rg` gate for a second engine/index root.
  5. Read-back of every edited wiki page to confirm frontmatter and record body survived (rule `durable-write-or-error`).

  ## Limitations

  - **Global-only rules are invisible to `wm_search`, the graph, and `wm_initial` page counts.** They are loaded by the session-start cascade only. Acceptable while the rule set is small and read in full every session; the upgrade path is shadow-with-provenance (D-rejected above).
  - **The cascade prevents new drift; it does not heal existing drift.** `no-comments-naming-only` and `no-comments-in-code` will not be recognised as the same intent by any machinery. Curation is human.
  - **A bad global edit binds every project at once.** Mitigated by footprint reporting, not by locks or transactions.
  - **Cascade surprise** is the top risk: an agent in a previously rule-less project begins being bound. Mitigated by the curation timing, the `wm_initial` counts line, and the opt-out.

  ## Rollback / Reintroduction

  `~/.wm/wiki/rules/` being a plain directory means the global set can be made a git repository later at near-zero cost, giving audit and rollback. Per-project divergence is always available through shadowing. Disabling the cascade is a one-key config change.

  ## Superseded (2026-10-15) — by @wiki/specs/global-wiki-layer

  The owner broadened the requirement: *"global layer should cover all wiki"* — every page type, not rules only. That meets the flip condition this spec's own design explicitly deferred: concepts, patterns, howtos and decisions are **searched, not read in full at session start**, so a read-time overlay outside the index leaves them invisible to `wm_search`, the graph and `wm_initial`.

  **Withdrawn:** the mechanism — read-time merge outside the index, rules-only, search-invisible.

  **Carried forward verbatim into @wiki/specs/global-wiki-layer:** the precedence rule (project wins by slug identity, reported rather than silent), the audit command (generalised to `wm-cli global audit`), move-semantics `promote` with collision refusal, the opt-out (`ignore_global_rules` → `ignore_global_wiki`), and the five-project curation plan.

  The rules-only reasoning below remains the historical record of why the cascade was shaped that way, and this spec's Evidence section is the origin of the superseding spec's measurement.
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