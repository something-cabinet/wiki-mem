---
title: Configurable state root — relocate .wm outside the project boundary
type: spec
id: "wiki:specs:configurable-wm-root"
status: approved
tags: [spec, config, relocation, paths, windows, security]
relates_to:
  - {type: relates_to, target: wiki:specs:remove-global-memory-layer}
---

schema_version: 1
state: |-
  ## Overview

  The engine currently assumes its state directory is `<project>/.wm`, and that assumption is structural rather than configurable: root discovery keys on the marker `<project>/.wm/config.json`, and every state consumer joins `WM_DIR` onto the project root. This spec makes the state directory **relocatable**: a project's `.wm` may live outside the project boundary, and several projects may share one.

  This is the same abstraction as @wiki/specs/global-wiki-layer at a different scope — *an engine consumes a `.wm` state dir that is not at the default location*. Built separately the two features would grow divergent root-resolution paths; built together, the ARCHITECTURE amendment stated by this spec survives unchanged.

  **Semantics chosen:** relocation and sharing — one mechanism, since sharing is simply N projects pointing at one target. **Rejected:** an additional project-configured read-only root, which is subsumed by the global layer's composed `$HOME/.wm` root and would reopen the "exactly two scopes" fence for no new capability.

  ## Evidence

  **Root discovery is positional and hardcoded** — `apps/wm-core/src/config/models/git_tracking_model.rs:56-74`: `WM_PROJECT` env check, else a CWD walk for `<dir>/.wm/config.json`, else `None`. `WM_PROJECT` overrides the *anchor* but the wiki must still be inside it.

  **No config key exists for a state location.** Current keys: `custom_edge_types, embedding, git_tracking, lsp, permissions, project_name, runtime_memory_max_entries, runtime_memory_recency_days, schema_version, search, source_dirs, source_extensions, status_colors, visible_columns`.

  **Confinement primitives already exist** (WM-003/WM-004, both closed): `confine` (`apps/wm-core/src/shared/helpers/path_confine_helper.rs:61`) and `confine_strict` (`:87`), plus `confine_under`/`confine_strict_under` (`shared/models/user_path_model.rs`), `confine_doc_path` (`mcp/tools/doc.rs:49`) and `confine_template_name` (`mcp/tools/template/mod.rs:12`).

  **A directly conflicting invariant exists.** `apps/wm-core/src/mcp/tools/lint.rs:206` emits `"type": "rogue_wm_dir"` — *"Found .wm/ directory outside project root... Only one .wm/ is allowed — at project root."* — backed by task `wiki:tasks:74454a` (done). The invariant conflated location with ownership; this feature separates them.

  **Call sites that assume `.wm` is under the project:** `audit_sink.rs:53`, `init_service.rs:165`, `freshness_service.rs:14`, `main_engine_factory.rs:97` and `:263`, `memory.rs:194`, plus the two `confine_*` helpers above.

  **Windows is in scope by construction, but untested.** `ci.yml:210` builds `x86_64-pc-windows-msvc` on `windows-latest` — that is the *release* matrix. Every check/test job (`ci.yml:24, :66, :93, :118, :143, :169`) runs on `ubuntu-latest`. The only two `#[cfg(windows)]` sites are test helpers (`tests/helpers/cli.rs:65`, `tests/helpers/mcp.rs:201`). So a Windows binary ships whose path-resolution code has never run under test.

  **The home resolver is copy-pasted in 10 places** — `model_service.rs`, `agents_service.rs`, `logging_helper.rs`, `detect.rs` (wm-lsp), `model.rs`, `main_engine_factory.rs`, `memory.rs`, `index.rs`, `manifest_store.rs`, plus one test. No shared helper. `memory.rs:194` is the canonical form: `HOME` → `USERPROFILE` → `"."`.

  ## Decision

  - **D1 — Relocation and sharing are one mechanism.** A project redirects its state directory via a pointer; N projects pointing at one target is the sharing case, not a separate feature.
  - **D2 — The anchor stays put.** `<root>/.wm/config.json` remains the discovery marker and `detect_project_root()` is **not modified** — `WM_PROJECT`, the CWD walk and the symlink handling from task `d7f99f` all keep working. In the relocated case that file is a **stub**: `{ "wm_root": "<path>" }` (plus optionally `project_name`) with no in-project `wiki/`.
  - **D3 — Root ≠ state root.** `detect_project_root()` still returns the project root; a new single accessor `state_root()` resolves the effective state directory. Every current `root.join(WM_DIR)` consumer becomes a `state_root()` consumer. One accessor is the DRY fix, and the migration is compiler-led.
  - **D4 — Two-tier config read.** The anchor config is read first (it must exist). If it carries `wm_root`, the config at `<wm_root>/config.json` is authoritative for all other keys. A `wm_root` key in the *resolved* config is a startup error — that kills redirect cycles and any "which file won" ambiguity.
  - **D5 — Hard errors, never silent fallback.** Dangling target, split state (anchor `wiki/` present *and* `wm_root` set), non-rooted value, filesystem-root value, home-exact value, and `wm_root` in the resolved config are each a startup error naming the constraint and the remediation. Silent fallback to the default `.wm` would split state — the worst available outcome.
  - **D6 — Trust gate with a canonical-path registry.** An unrecognised `wm_root` target is a startup error until approved once via `wm config accept-root <path>`. Without this, a cloned repository could carry a stub pointing anywhere writable and every wiki write would follow. The registry lives in `~/.wm/projects.json` (introduced by this spec) as an approved-roots list of canonical expanded paths plus timestamps. Canonicalisation precedes the gate, so two spellings of one target share one approval. Every redirect resolution is audit-logged.
  - **D7 — Two confinement domains**, derived once at engine construction and immutable for the process: (1) the **project root** for source ingestion (`source_dirs`, `source_extensions`), unchanged; (2) the **resolved state root** for wiki pages, docs, templates, memory and the audit log. A third domain for a global root existed only under the removed global-wiki-layer spec and is withdrawn with it. The `confine`/`confine_strict` primitives are untouched — they simply receive the resolved root, so `..` traversal and symlink escapes inside the wiki are rejected exactly as WM-003/WM-004 enforced them.
  - **D8 — `rogue_wm_dir` is amended, not waived, and gets stronger.** The new rule: every `.wm` must be accounted for as (a) a discovered project's default state dir, (b) a `wm_root` target referenced by a registered stub, or (c) `$HOME/.wm`. Stub-form `.wm` is whitelisted. A **stale in-project `.wm` left behind after relocation is rogue** — that catches half-migrations in CI instead of silently splitting state. Orphaned external state dirs (pointer removed, data abandoned) are rogue via registry scan. Task `74454a` stays done; a new task amends the lint's allowlist from positional to registry-derived.
  - **D9 — Precedence chain, outright.** `WM_PROJECT` (anchor selection) → anchor `config.json:wm_root` → default `<root>/.wm`. Never merged, never fallen back from; every non-default resolution is audit-logged. There is deliberately **no env override for `wm_root`**: CI isolation already has `WM_PROJECT` for the anchor and `HOME`/`USERPROFILE` for the global root, and a third knob is YAGNI.
  - **D10 — `relocate` and rollback use move semantics with staged safety.** `wm config relocate --to <path>`: refuse a non-empty target → copy → verify (file count plus size/hash spot-check) → write the stub → **only then** remove the original. The stub is not declared until the payload is verified, so an induced failure leaves the original intact and no stub written (`durable-write-or-error`). Rollback is `wm config relocate --to .`. There is no copy-and-keep-both variant: two live state dirs is precisely the split-state failure this design exists to prevent.
  - **D11 — Single state root; no composition.** There is no multi-root composition and no global scope. A configured root simply **is** the project's state root: the engine stays rooted in exactly one directory, which is either the default `<root>/.wm` or the `wm_root` target. The ARCHITECTURE single-root invariant continues to hold literally and needs no amendment for this spec. The ARCHITECTURE amendment gains one clarifying sentence: *"The engine composes scopes; each scope resolves to exactly one root; there are exactly two scopes."*
  - **D12 — One home resolution per platform, verified, never a chain.** The flaw in `HOME` → `USERPROFILE` → `"."` is not only the `"."` — a chain makes the location **shell-dependent**. On Windows, `HOME` is set by Git Bash/MSYS but not by PowerShell or cmd, so the same project would resolve its state directory differently depending on which terminal launched the agent. For a store that must be found again tomorrow by a different process, that is an unrepeatable-location bug, worse than any single wrong answer. Therefore:

    | Platform | Authority | Behaviour |
    |---|---|---|
    | Unix / macOS | `HOME` only | unset, empty or unverifiable → hard error |
    | Windows | `USERPROFILE` only | if unset, probe `HOMEDRIVE`+`HOMEPATH` **only when the concatenation exists and is a directory**; else hard error |

    `HOME` is **ignored on Windows even when set** — disagreement becomes unreachable, so determinism beats deference to the launching shell. Verification is part of resolution: the result must exist and be a directory, which guards against a stale `USERPROFILE` after a profile rename and against `HOMEPATH` junk on domain-joined machines. `LOCALAPPDATA` is rejected as a home substitute — it is for app-local caches, not user knowledge, and the global scope already standardised on `$HOME/.wm`.
  - **D13 — The `"."` fallback dies everywhere.** Every copy of the chain is replaced by a single `home_dir() -> Result<PathBuf, ConfigError>` whose error payload names the platform, the variable consulted, the probe result and the remediation. **No fallback past the platform authority, ever.** Pre-feature, a `"."` resolution produced a weird in-project `wiki/` and got noticed; post-feature the resolved path can be a write target *outside* the project, so a silent `"."` now means "the wiki is wherever the agent's CWD happened to be" — including `/`, a random repository, or a scratch directory. The `memory.rs:194` copy disappears with @wiki/specs/remove-global-memory-layer; the remaining copies are a `USERPROFILE` grep sweep, deleted rather than suppressed (`no-dead-code-clone-scanning`).
  - **D14 — One helper, in the constants layer.** `home_dir()` and `expand_user_path()` live beside `WM_DIR`/`WIKI_DIR`. Beyond DRY, the path that locates all state is the most security- and correctness-critical path in the binary, and eleven subtly divergent copies means eleven places a trust gate can be bypassed by accident. One function, one audit point, one test suite; the sweep is compiler-verifiable.
  - **D15 — Config storage grammar: exactly two accepted forms.** `~/path/to/dir` (home-relative, expanded at load via D12's function) or an absolute path in platform form. Rejected, each as a startup error: bare `~` (profile-exact), bare relative paths, drive-relative `C:foo` — which is the `"."` bug wearing a drive letter, and is caught by a `Path::has_root()` check — and env-var interpolation such as `%APPDATA%` or `$VAR` (no second expansion language; YAGNI). Forward slashes are mandated in stored values so stubs diff cleanly cross-platform; `std::path` normalises them on Windows without platform branching in our code. Symbolic `~/` storage is deliberate rather than absolute-only: an absolute stub would leak profile names into git and break the family-branch sharing case the moment a second user or machine checks out the repository.
  - **D16 — Windows path correctness lives inside the confine helpers, once.** Canonicalisation on Windows yields extended-length `\\?\` verbatim prefixes; if a canonicalised candidate is compared against a non-canonicalised or differently-prefixed root, the prefix check misses and confinement either over- or under-rejects. Both root and candidate are therefore canonicalised and prefix-normalised (`\\?\` stripped) *inside* `confine`/`confine_strict`, not at call sites. Windows comparisons must also be case-insensitive-aware, since canonicalisation does not normalise case. These belong as regression tests in the WM-003/WM-004 fixture style.

  ### Rejected alternatives

  - **An additional project-configured read-only root.** Subsumed by the global layer's composed `$HOME/.wm` root; a third project root would reopen the "exactly two scopes" fence for no new capability.
  - **Modifying `detect_project_root()` to return the state dir.** Root and state root are distinct concepts; conflating them breaks source ingestion confinement (D7 domain 1) and the existing `WM_PROJECT` semantics.
  - **Silent fallback to the default `.wm` when the target is missing.** Splits state — the worst available outcome, and exactly the class `no-compensating-layers` forbids.
  - **Auto-trusting a `wm_root` target.** A cloned repository's stub would redirect every wiki write to an attacker-chosen writable path.
  - **Copy-and-keep-both relocation.** Guarantees two live state dirs and drift.
  - **A `wm_root` env override.** Redundant with `WM_PROJECT` and the home variables.
  - **`HOME`-first precedence on Windows.** Wrong on Windows, where `HOME` may hold an MSYS-style POSIX path from Git Bash; `USERPROFILE` is the native authority.
  - **`LOCALAPPDATA` as the Windows home.** Splits semantic "home" from where users expect `$HOME/.wm`, and the global scope already standardised on the profile directory.

  ## ARCHITECTURE amendment (addition)

  In addition to the two-scope sentence proposed by @wiki/specs/global-wiki-layer, record:

  > State-root and home resolution are single functions with defined failure semantics; no resolution path may fall back to the current working directory.

  ## Contract

  **Anchor.** `<root>/.wm/config.json`, unchanged as the discovery marker.

  **Redirect.** `{ "wm_root": "~/path" | "<absolute>" }` in the anchor config; authoritative config at `<wm_root>/config.json`.

  **Resolution.** `state_root()` — the single accessor replacing every `root.join(WM_DIR)`.

  **Precedence.** `WM_PROJECT` → anchor `wm_root` → default `<root>/.wm`; never merged, never fallen back from.

  **Approval.** `wm config accept-root <path>`; registry `~/.wm/projects.json`.

  **Verbs.** `wm config relocate --to <path>`; rollback `--to .`; `wm init --wm-root <path>`.

  **Confinement.** `confine`/`confine_strict` receive the resolved state root for domain 2; canonicalised, prefix-normalised and case-normalised internally.

  ## Change Inventory

  **Engine**
  - New `state_root()` accessor; every `root.join(WM_DIR)` consumer migrated (`audit_sink.rs:53`, `init_service.rs:165`, `freshness_service.rs:14`, `main_engine_factory.rs:97,263`, `doc.rs:49`, `template/mod.rs:12`).
  - Two-tier config read with the redirect validation chain.
  - Two confinement domains derived at engine construction; `confine`/`confine_strict` updated for canonicalisation, `\\?\` stripping and case-insensitive comparison.
  - Trust gate + approved-roots registry read from `~/.wm/projects.json`; redirect resolution audit-logged.

  **Constants layer**
  - `home_dir() -> Result<PathBuf, ConfigError>` and `expand_user_path()`, beside `WM_DIR`/`WIKI_DIR`.
  - Delete the ten duplicated `HOME`/`USERPROFILE` resolutions (compiler-led sweep).

  **CLI / MCP**
  - `wm config accept-root`, `wm config relocate` (staged move), `wm init --wm-root`.
  - Amended `rogue_wm_dir` lint with a registry-derived allowlist.

  **Docs and embedded assets (RustEmbed'd — change in lockstep with repo-root twins)**
  - `wiki:core:ARCHITECTURE` — adopt the amendment sentence above alongside the two-scope sentence.
  - `wiki:core:CONVENTIONS` — document the pointer model, the config grammar and the home-resolution contract.
  - `WIKI-MEM.md` + the six shim twins + repo-root `WIKI-MEM.md` — describe the pointer model and the family-branch recipe.
  - `wm-init` skill in both copies, coordinated with @wiki/specs/remove-global-memory-layer and @wiki/specs/global-wiki-layer

  ## Migration

  Existing projects are unaffected: no `wm_root` key means today's behaviour, and AC-1 pins that. To adopt relocation, run `wm config relocate --to <path>`, which stages copy → verify → stub → remove so the original survives any induced failure. The family-branch recipe is: initialise once, then stub each sibling worktree at the shared target — the configuration is single-sourced at the target, so sibling stubs **cannot disagree**, because the stub is pointer-only.

  ## Acceptance Criteria

  - [ ] AC-1: `detect_project_root()` behaviour is unchanged when no `wm_root` is present (the existing suite passes unmodified).
  - [ ] AC-2: A stub `{ "wm_root": "..." }` causes the engine to compose the resolved root; search, graph, page CRUD and the audit sink all operate on it.
  - [ ] AC-3: Six distinct startup errors — dangling target, split state, non-rooted value, filesystem-root value, home-exact value, and `wm_root` present in the resolved config — each naming its constraint, with zero silent fallbacks.
  - [ ] AC-4: An unapproved target errors naming `wm config accept-root`; after approval it succeeds and the approval is recorded in the registry and audit-logged.
  - [ ] AC-5: Two stubs pointing at one target produce an identical index in both projects; multi-project pointers are surfaced by `wm-cli global audit`.
  - [ ] AC-6: The approved-roots registry is self-contained in this spec - no other spec introduces it - and any `wm_root` target is gated solely by `wm config accept-root`.
  - [ ] AC-7: Confinement regression — `../../` and symlink-escape writes against a relocated root are rejected by the existing `confine_*` primitives rooted at the resolved directory (mirrors the WM-003/WM-004 fixtures).
  - [ ] AC-8: Amended lint — a stale post-relocation in-project `.wm` is flagged rogue; stub-form `.wm` is whitelisted; registry-referenced external state dirs are whitelisted; an unowned external `.wm` remains rogue.
  - [ ] AC-9: `relocate` ordering is copy → verify → stub → remove, tested with an induced I/O failure that leaves the original intact and no stub written.
  - [ ] AC-10: `embed_files/` and the repo-root twins describe the pointer model identically (byte-compared in lint), and the family-branch recipe is documented.
  - [ ] AC-11: Exactly one `EngineStateMediator`; zero new dependencies; `cargo build`/`clippy` warning-free; no `#[allow(dead_code)]`.
  - [ ] AC-12: A **Windows CI fixture** (windows-latest or equivalent) exercises, against the real APIs: `USERPROFILE` normal; `HOME` set but ignored on Windows (the cross-shell determinism test); `USERPROFILE` unset with a valid `HOMEDRIVE`+`HOMEPATH` probe; all-unset producing the hard-error payload shape; `~/` expansion; drive-relative `C:foo` rejection; UNC canonicalisation with `\\?\`-stripped confine comparison; mixed-separator config round-trip; and a case-differing prefix confine test.
  - [ ] AC-13: If no Windows runner exists at merge time, the *Limitations* section must state plainly — not as a footnote — that all Windows path claims are unverified and the feature is macOS/Linux-verified only, and the `wm_root` documentation must say the same. An unverified claim shipped as fact is the documentation-level form of the `durable-write-or-error` violation.

  ## Verification

  1. Startup-error matrix from AC-3 exercised as integration tests, asserting the error payload names the constraint.
  2. Confinement regression suite mirroring the WM-003/WM-004 fixtures, plus the new Windows canonicalisation/case cases.
  3. Amended-lint fixtures for AC-8 (stub-form, stale, registry-referenced, unowned).
  4. Two-stub shared-target test producing identical indexes.
  5. Home-resolution unit tests per platform, including the all-unset error payload.
  6. Read-back of every edited wiki page to confirm frontmatter and record body survived (rule `durable-write-or-error`).

  ## Limitations

  - **Windows path claims are unverified today.** No Windows check/test job exists — only the release matrix builds `x86_64-pc-windows-msvc` (`ci.yml:210`). Until AC-12 is satisfied, every Windows statement in this spec is unproven, and AC-13 governs how it must be described.
  - **Shared-state concurrency** across family-branch checkouts relies on the existing watcher/debounce model. Last-writer-wins at the filesystem level; flip signal is observed lost updates, which would call for per-file advisory locking — never IPC or a database.
  - **Trust-gate friction** may burden legitimate flows such as CI matrices spinning fresh roots; the remedy would be `--approve-root` on init or an approved-roots environment allowlist, never auto-trust.
  - **Stub/resolved config drift is structurally impossible** only because the stub is pointer-only. Any future request for per-checkout overrides of resolved keys reopens the disagreement class this design closes and must be refused.
  - **Scope creep toward N roots.** "Exactly two scopes" is the fence. A future team-scoped root between project and global would need sync and consensus semantics that markdown directories do not have, and deserves its own RFC.

  ## Rollback

  `wm config relocate --to .` moves state back and removes the stub; the registry entry persists harmlessly. Removing the `wm_root` key restores default resolution. Deleting the target without relocating first is a startup error, not a silent fallback, which keeps the failure visible and recoverable.

  ## Amendment (2026-10-15) - composition-dependent parts withdrawn

  The global wiki layer was removed by owner decision (see the Superseded note on wiki:specs:global-wiki-layer). This spec survives because relocating a single project's state root is independent of cross-project sharing, but several parts were written against the composition and are withdrawn:

  - The **global scope** and the `$HOME/.wm/wiki` root: gone. There is no second scope and no composed root.
  - The **third confinement domain** (global root): withdrawn; two domains remain.
  - The **scope/provenance contract**, **shadow reporting**, **closed shareable set**, **promote/demote** and the **global audit command**: all withdrawn with the composition.
  - The **approved-roots registry** is introduced by this spec rather than by the global-layer spec.
  - **Sequencing:** this spec no longer needs to land alongside any other. It is standalone.

  **Unaffected, and still the point of the spec:** the pointer stub, the `state_root()` accessor, the two-tier config read, the six startup errors, per-platform home resolution with no `.` fallback, the single shared `home_dir()` helper, the config grammar, Windows canonicalisation inside the confine helpers, the trust gate, `wm config relocate` and rollback, the `rogue_wm_dir` amendment, the Windows CI fixture, and the family-branch recipe.
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