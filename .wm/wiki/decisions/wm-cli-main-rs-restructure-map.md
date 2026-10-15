---
title: wm-cli main.rs Restructure Map (D1)
type: decision
id: "wiki:decisions:wm-cli-main-rs-restructure-map"
status: approved
tags: [rust, wm-cli, architecture, module-map, controlled-rewrite]
---

schema_version: 1
state: |-
  ## Status

  **Approved 2026-10-15** after nine review waves (R1-R9). Scope: `apps/wm-cli` only. No changes to `wm-core` or `packages/*`.

  ## Context

  `apps/wm-cli/src/main.rs` was the largest `.rs` file in the workspace: 3,058 lines, 48% of the crate, with a single `async fn main()` spanning 2,061 lines across 21 top-level command arms, 15 clap enums, 2 structs and 231 print macros. It is now **53 lines** plus 25 `mod` declarations, and no file exceeds the budget.

  ## Constraints

  1. One primary type per file; exception is tightly coupled helpers under 20 lines.
  2. Role-suffixed filenames in role directories; every directory gets a `mod.rs` barrel. **Role directories live INSIDE feature directories** — the repo shape is `wm-core/src/page/{helpers,services}`, `wm-core/src/skill/{models,services,helpers}`; a root role directory containing feature directories inverts it.
  3. Shared-code rule, refined by C8.
  4. `no-comments-in-code` — zero comments including rustdoc.
  5. `no-warnings` and `no-dead-code-clone-scanning` after every step.
  6. `no-compensating-layers`, `no-else`, `no-magic-values`, YAGNI.
  7. TDD; **behaviour freeze**: command names, flags, help text, stdout and exit codes unchanged.
  8. **C-share**: an item with 2 or more consumers is shared, counted at call sites. One consumer means it stays with its consumer, except where an exception is recorded.
  9. **C-decompose**: a function whose behaviour is not self-evident from name and structure is a defect. Functions target 20 lines. **The cap must never drive a worse idiom** (R8).
  10. **C-domain**: a domain owns its own model and behaviour in one directory, and **must not import a sibling domain**. Cross-domain needs go to `shared/`. Only the router may know every domain.

  ## Decision - target module map

  ```
  apps/wm-cli/src/
    main.rs                    setup_logging, parse, help fallback, 21-arm dispatch
    constants/                 fs, health, wiki (+ mod.rs)
    models/                    output_format (+ mod.rs)
    mcp_server.rs              out of scope
    shared/
      models/    cli_model.rs, commands_model.rs   ONLY the crate-wide clap surface
      traits/    command.rs                        the Command contract
      services/  engine_service, engine_proxy, agent_file_sync_service, platform_mcp_service
      helpers/   logging_helper, project_root_helper
    <domain>/                  21 FLAT feature directories at the crate root
      mod.rs
      <domain>_service.rs      behaviour (impl Command for grouped domains)
      <domain>_args_model.rs   leaf domains only
      <domain>_action_model.rs grouped domains only
      <domain>_renderer.rs     page, graph
      <domain>_helper.rs       health, init
  ```

  The 21 domains: agents, config, decide, graph, health, index, init, lint, log, mcp, migrate_memory, model, page, search, setup, source, status, task, time, validate, version.

  Dependency direction: `<domain>` -> `shared` -> `wm-core`, `constants`. The single reverse edge is the router, which references all 21 domains because it is the composition root.

  ## Superseded layout (recorded so it is not reintroduced)

  The first version of this decision nested the domains inside a root `services/` role directory (`services/<domain>/`), with per-leaf args models and action enums in `shared/models/`. It was **rejected** because it inverted the repo shape (constraint 2) and split every domain across two homes — the service under `services/<domain>/` and its own model under `shared/models/`. That is the same two-homes defect for which an earlier option was rejected. Commit `5cfba93` moved the domains to the crate root and moved each domain`s models into its own directory; `shared/models/` now holds only `Cli` and `Commands`.

  ## Placement

  | Item | Destination | Basis |
  |---|---|---|
  | `trait Command` | `shared/traits/command.rs` (unsuffixed, named after the trait, mirroring wm-core) | 21 consumers |
  | `impl Command` x 21 | inside each domain | behaviour lives with the domain |
  | `Cli`, `Commands` | `shared/models/` | the crate-wide clap surface, consumed by `main` and the router |
  | per-domain action enums and leaf args models | inside their own domain | C-domain: a domain owns its model |
  | `McpTransport` -> `mcp/`, `ExportFormat` -> `graph/` | their domain | they are fields of `McpArgs` and `GraphAction`, not crate-wide |
  | `call_tool` (31 call sites), `EngineHandle`, `engine_handle` | `shared/services/engine_proxy.rs` | C-share; `_proxy` denotes lazy init and access control |
  | `create_engine`, `build_engine`, `rebuild_from_engine` | `shared/services/engine_service.rs` | 2-3 consumers each |
  | `sync_agent_files`, `sync_skills_to` | `shared/services/agent_file_sync_service.rs` | 3 and 6 consumers |
  | `setup_platform_mcp`, `resolve_mcp_binary`, `patch_mcp_command` | `shared/services/platform_mcp_service.rs` | 2, 2, 5 consumers |
  | `setup_query_before_grep` | `setup/setup_service.rs` | 1 consumer |
  | `determine_project_root`, `setup_logging` | `shared/helpers/` | **recorded C-share exceptions** (1 call site each) |
  | `health_status_label` | `health/health_service.rs` (private) | 1 consumer |
  | the `AGENTS.md` template | `init/init_service.rs` | data, not logic; relocation is D3 |

  ## Known two-homes remnants (follow-up, found by measurement)

  | Item | Finding |
  |---|---|
  | `models/output_format.rs` | referenced by exactly one domain (`health`) — under C-share it belongs in `health/` |
  | `constants/health.rs` | 2 of its 3 constants are health-specific and belong in `health/`; `DEFAULT_EDGE_TYPE` is a graph/refs constant mislocated under `health.rs` |
  | `constants/{fs,wiki}.rs` | 4 references, all from the health domain — needs a wider usage check before deciding |

  ## Decomposition mandate

  Every function targets 20 lines or fewer, single responsibility, verb+object naming, intent evident without comments.

  Named exceptions: the `AGENTS.md` raw string (~110, data not logic); `dispatch` (~25, flat match, complexity about 1); the `Commands` enum (a type); and `run_page_action` (28) with `run_graph_action` (20) — flat per-domain dispatchers, exempted rather than satisfied with panic sites (R8).

  ## D8 - dispatch and the Command trait

  `main()` is `setup_logging()`, `Cli::parse()`, the help fallback, then a 21-arm exhaustive `match` delegating to `run`. `dispatch` moves to `command_dispatch_service.rs` in step 4.

  ```rust
  pub trait Command {
      fn run(self) -> impl Future<Output = Result<(), anyhow::Error>> + Send;
  }
  ```

  - Implemented by all 21 domains: 12 grouped domains through their action enum, 9 leaf domains through an args struct. Leaf variants are single-field tuple variants, so all 21 arms read identically.
  - The explicit `+ Send` bound is deliberate: bare `async fn` in a trait carries no `Send` bound and would break a future `tokio::spawn`. It also removes the sync/async problem, because four domains never await.
  - Static dispatch only: monomorphised, no `Box`, no vtable, no allocation. A registry or `Box<dyn>` dispatch is **rejected** — the command set is a closed enum the compiler enumerates exhaustively.
  - The exhaustive `match` **remains**: it converts `Commands` into the concrete type, which is where exhaustiveness is enforced.
  - **Revisit trigger**: if the command set becomes dynamic or plugin-loaded, the factory precondition appears and D8 reopens.
  - The trait must not exist before its first impl: `wm-cli` is a binary crate, so a `pub trait` with no implementors is dead code.
  - Naming recorded: `std::process::Command` is used only inside the init domain, fully qualified and never imported, so it cannot collide with the trait; keep it that way.

  ## Verification of C-domain

  Measured on the committed tree after `5cfba93`: **0 cross-domain imports**. Every domain points at `shared/{models,traits,services,helpers}` and `wm-core`; intra-domain `super::<own>_helper` stays inside its own directory. This is the invariant a future check should enforce.

  ## Size budget and enforcement

  **300 lines per file, `main.rs` at most 80, functions at most 20 (named exceptions above).** Measured largest files: health_service 281, health_audit_helper 249, page_service 240, search_service 230, init_service 215, init_wizard_helper 208, graph_renderer 187. `main.rs` is **73 lines** — the 21 `mod` declarations consumed most of the remaining headroom against the 80-line cap; if a 22nd domain arrives, the cap or the bin/lib split must be revisited rather than exceeded.

  Enforced by a deterministic CI check in the `check` job of `.github/workflows/ci.yml`, following the `#[allow(...)]`-ban precedent. Note for future checks: an `-exec` loop body must not end in an `&&` list — that returns non-zero for every passing file, propagates through `find`, and reds the step under `bash -e`.

  ## Explicit non-decisions

  | # | Topic | State |
  |---|---|---|
  | D2 | clap surface mechanism | types placed with derives intact; mechanism later |
  | D3 | `AGENTS.md` template | inline with the init domain; relocation later |
  | D4 | `EngineHandle` and `OnceLock` lifecycle | located in `shared/services/engine_proxy.rs`; **must reuse or extend** `apps/wm-core/src/engine/main_engine_factory.rs` — no second construction path |
  | D5 | agent-sync boundary | two shared files today; finer split later |
  | D6 | output layer for the 229 `println!` | per-domain `print_*` / renderer functions; abstraction later |
  | D7 | `anyhow` versus typed errors | preserved verbatim |
  | - | per-domain construction | a `*_builder_service.rs` inside the domain, never a central factory |
  | - | `traits/` role-directory gap | `traits/` is not in CONVENTIONS` documented role list although `shared/traits/` exists; amend CONVENTIONS or document the exception |

  ## Migration ordering

  | Step | Content | State |
  |---|---|---|
  | 0 | Characterization net: 33 golden fixtures (binary help, all 21 subcommand helps, read-only stdout, error paths) generated from the pre-move binary, plus a harness that fails loudly when the binary is stale | DONE |
  | 1a | Move the 16 clap surface types into `shared/models/` verbatim | DONE |
  | 1b | Per-leaf args structs and tuple variants | DONE |
  | 2 | `shared/helpers/` and the shared services | DONE |
  | 3 | 21 per-domain commits: directory, verbatim move, decompose to 20-line functions, one-line delegation | DONE |
  | 3b | Move domains to crate-root feature directories; per-domain models into their domains; fix two size breaches; land the CI budget check | DONE (`5cfba93`) |
  | 4 | Extract `dispatch` to `command_dispatch_service.rs`, add in-process routing tests | OPEN (a decision: `main.rs` is already 73 lines, so this now buys testability rather than slimming) |
  | 5 | Re-home `mcp_server.rs`; drop the unused `rayon` and `chrono` dependencies | OPEN |

  ## Risks

  | Risk | Consequence | Mitigation |
  |---|---|---|
  | Decomposition drifts stdout or exit codes | behaviour freeze broken | per-leaf goldens, two-phase commits, bottom-up extraction |
  | A cap-driven idiom change sneaks in | new panic paths in a frozen refactor | R8: the cap never justifies a worse idiom |
  | A domain reaches into a sibling | domain boundaries become nominal | C-domain plus a future deterministic check |
  | The crate grew ~25% in total lines | larger surface despite a smaller `main.rs` | accepted: the cost of named 20-line functions, per-domain printers and args structs |

  ## Reviewer amendments incorporated

  | Wave | Amendment |
  |---|---|
  | R1 | Uniform one directory per command, all 21; no inline arms; health decomposed in-dir; budget as a CI check |
  | R2 | Shared module first-class |
  | R3 | Share means 2 or more consumers; decomposition mandate |
  | R4 | Dispatch is an extracted exhaustive match, not a factory; D4 reuses `main_engine_factory.rs` |
  | R5 | The `Command` trait is mandatory and shared, unsuffixed, impls in each domain |
  | R6 | Trait signature RPITIT with explicit `+ Send`; 21/21 coverage via per-leaf args structs; `shared/helpers/` promoted as an exception |
  | R7 | Both helpers promoted; step 1 split into 1a and 1b |
  | R8 | The two flat per-domain dispatchers are exempt from the 20-line cap, and the 14 `unreachable!()` sites introduced to satisfy it are rejected and removed. Binding: the cap never drives a worse idiom |
  | R9 | **Domains are feature directories at the crate root, not nested under a root role directory.** A domain owns its model and behaviour (no two homes), and must not import a sibling domain. The superseded `services/<domain>/` layout is recorded above so it is not reintroduced |

  ## Open recommendations

  1. Create `@wiki/rules/file-size-budget` (one invariant plus a pointer to the CI step). Requires reviewer approval.
  2. Land the unused `rayon` and `chrono` removal as a separate commit.
  3. Amend CONVENTIONS to add `traits/` to the role-directory list, or document `shared/traits/` as the accepted exception.
  4. Fix the three two-homes remnants listed above, and add a deterministic check that no domain imports a sibling domain.
  5. Structure reviews are filed for the sibling crates: `apps/wm-core`, `packages/wm-code-intel`, `wm-engine`, `wm-embed`, `wm-search`. `wm-lsp` and `wm-constants` measured healthy.
questions:
  - id: outcome
    type: choice
    instructions: What is the recorded outcome of this decision?
    options:
    - adopted
    - rejected
    - deferred
    - superseded
    - abandoned
  - id: reversibility
    type: noul
    instructions: The decision can be reversed cheaply without data migration or cross-module breakage.
  - id: confidence
    type: score
    instructions: How strong is the recorded justification for the selected outcome?
    levels:
    - low
    - medium
    - high
  - id: impact
    type: choice
    instructions: How wide is the blast radius of this decision?
    options:
    - local
    - component
    - system
    - project-wide
answers:
  outcome: adopted
  reversibility: false
  confidence: high
  impact: component
