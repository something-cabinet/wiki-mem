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

  **Approved 2026-10-15** after eight review waves (R1-R8). Scope: `apps/wm-cli` only. No changes to `wm-core` or `packages/*`.

  ## Context

  `apps/wm-cli/src/main.rs` was the largest `.rs` file in the workspace: 3,058 lines (next-largest non-test file 2,190), 48% of the crate, with `async fn main()` spanning 2,061 lines across 21 top-level `Commands` arms and about 50 leaf actions, 15 clap enums, 2 structs, 16 functions, 231 print macros, and a 4,986-byte inline `AGENTS.md` template inside the Init arm.

  Existing structure preserved as precedent: `src/constants/` (`fs`, `health`, `wiki`, `mod`), `src/models/` (`output_format`, `mod`), root `src/mcp_server.rs`.

  ## Constraints

  1. One primary type per file; exception is tightly coupled helpers under 20 lines.
  2. Role-suffixed filenames in role directories (`models/`, `services/`, `helpers/`, `constants/`); every directory gets a `mod.rs` barrel. **Gap flagged**: `traits/` is not in the documented role-directory list although `apps/wm-core/src/shared/traits/` exists.
  3. Shared-code rule, refined by C8.
  4. `no-comments-in-code` — zero comments including rustdoc.
  5. `no-warnings` and `no-dead-code-clone-scanning` after every step (including `unused_async`).
  6. `no-compensating-layers`, `no-else`, `no-magic-values`, YAGNI.
  7. TDD; **behaviour freeze**: command names, flags, help text, stdout and exit codes unchanged.
  8. **C-share**: an item is shared when it has **2 or more consumers**, counted mechanically at call sites, including items that only become shared after the split. One consumer means it stays with its consumer, **except** where the reviewer explicitly records an exception (see `shared/helpers/`).
  9. **C-decompose**: a function whose behaviour is not self-evident from name and structure is a defect. Every function targets 20 lines or fewer; verbatim lift is not sufficient. **The cap must never drive a worse idiom** — see the exception list, and R8.

  ## Options considered

  | Option | Tree | Verdict |
  |---|---|---|
  | **A - Role dirs + uniform per-command dirs + first-class `shared/`** (chosen) | `shared/{traits,models,services,helpers}/`; 21 `services/<command>/` dirs; one router file | Zero rule violations; uniform granularity; satisfies C-share mechanically; mirrors `wm-core/src/shared/` |
  | B - Feature-first `commands/<name>/` | non-sanctioned directory; `Cli`/`Commands` homeless | Rejected |
  | C - Hybrid `commands/` plus `models/` | two homes per feature | Rejected |
  | D - Minimal concern split | ~2,300-line `main.rs` remains | Rejected: fails the goal and the budget |
  | E - Dispatch registry or factory | trait object plus `Box<dyn>` plus dynamic dispatch | **Rejected, see D8** |

  ## Decision - target module map

  ```
  apps/wm-cli/src/
    main.rs                    setup_logging call, parse, help fallback, dispatch
    constants/                 unchanged (fs, health, wiki, mod)
    models/                    unchanged (output_format.rs, mod.rs)
    mcp_server.rs              out of scope
    shared/
      mod.rs
      traits/  mod.rs, command.rs
      models/  25 files + mod.rs: Cli, Commands, 14 action/support enums, 9 leaf args structs
      services/  mod.rs, engine_service.rs, engine_proxy.rs, agent_file_sync_service.rs, platform_mcp_service.rs
      helpers/  mod.rs, logging_helper.rs, project_root_helper.rs   [both promoted as recorded exceptions]
    services/
      mod.rs
      command_dispatch_service.rs                            (step 4)
      <command>/{mod.rs, <command>_service.rs}               x 21 uniform directories
  ```

  The 21 command directories: init, mcp, setup, agents, search, page, graph, health, source, task, log, lint, validate, index, time, model, decide, status, config, version, migrate_memory.

  Dependency direction: command directories depend on `shared`, which depends on `wm-core` and `constants`. The single reverse edge is the router, which references all 21 command directories because it is the composition root.

  ## Placement

  | Item | Destination | Basis |
  |---|---|---|
  | `trait Command` | `shared/traits/command.rs` (unsuffixed, named after the trait, mirroring wm-core) | 21 consumers |
  | `impl Command` x 21 | inside each command module | impls live beside the behaviour they implement |
  | `Cli`, `Commands`, 14 action enums, 9 leaf args structs | `shared/models/` | consumed by the dispatch; recorded structural exception to C-share for `Cli`/`Commands` |
  | `call_tool` (31 call sites), `EngineHandle`, `engine_handle` | `shared/services/engine_proxy.rs` | C-share; `_proxy` denotes lazy init and access control |
  | `create_engine`, `build_engine`, `rebuild_from_engine` | `shared/services/engine_service.rs` | 2-3 consumers each |
  | `sync_agent_files`, `sync_skills_to` | `shared/services/agent_file_sync_service.rs` | 3 and 6 consumers |
  | `setup_platform_mcp`, `resolve_mcp_binary`, `patch_mcp_command` | `shared/services/platform_mcp_service.rs` | 2, 2, 5 consumers |
  | `setup_query_before_grep` | `services/setup/setup_service.rs` | 1 consumer, stays with Setup |
  | `determine_project_root` | `shared/helpers/project_root_helper.rs` | **promoted as a recorded C-share exception** (1 call site) |
  | `setup_logging` | `shared/helpers/logging_helper.rs` | **promoted as a recorded C-share exception** (1 call site) |
  | `health_status_label` | `services/health/health_service.rs` (private) | moves with Health |
  | `AGENTS.md` template | travels with the Init arm | data, not logic; final home is D3 |

  ## Decomposition mandate

  Every function targets 20 lines or fewer, single responsibility, verb+object naming, intent evident without comments. All 21 commands implement `Command`, so all 21 dispatch arms read identically.

  Named exceptions to the 20-line target, each justified:

  | Item | Why |
  |---|---|
  | the `AGENTS.md` raw string (~110) | data, not logic; frozen for behaviour fidelity |
  | the router `dispatch` (~25) | flat match, one statement per arm, cyclomatic complexity about 1; splitting into read/write groups would be artificial |
  | the `Commands` enum (~125) | a type, not a function |
  | `run_page_action` (~24-28) and `run_graph_action` (~24-28) | the same shape as `dispatch`: a flat per-command match, one statement per arm, complexity about 1. Only `Page` and `Graph` variant field lists are long enough that rustfmt expands inline patterns past the cap. **Exempted rather than satisfied by adding panic sites** |

  **R8 precedent, binding for the rest of the rewrite:** the cap must not drive a worse idiom. The first implementation of `page` and `graph` met the cap by having each leaf re-destructure the whole action with `let ... else { unreachable!() }`, introducing 14 panic sites that the original code did not have, in a behaviour-frozen refactor, and with no precedent anywhere in the repository. That was rejected. Where a flat match would breach the cap, the dispatcher is exempted and leaves receive the narrowed fields as typed parameters, matching the sibling services.

  ## D8 - dispatch and the Command trait

  **Decided.** `main()` becomes `setup_logging()` then `Cli::parse()` then the help fallback then `dispatch(cli.command).await`. `dispatch` is an exhaustive `match` over `Commands` with 21 arms, living in `services/command_dispatch_service.rs`.

  **The trait, shared, static dispatch:**

  ```rust
  pub trait Command {
      fn run(self) -> impl Future<Output = Result<(), anyhow::Error>> + Send;
  }
  ```

  - Implemented by all 21 commands: the 12 grouped commands through their Action enum (`impl Command for SearchAction`), the 9 leaf commands through a per-leaf args struct (`impl Command for InitArgs`).
  - Leaf variants are single-field tuple variants carrying their args struct, so every arm is identical: `Commands::Search { action } => action.run().await` and `Commands::Init(args) => args.run().await`.
  - The explicit `+ Send` bound is deliberate: bare `async fn` in a trait gives no `Send` bound by default, which would break any future `tokio::spawn`. It also removes the sync/async problem, because four commands (Health, Model, Log, Config) never `.await` and would otherwise trip clippy`s `unused_async`.
  - Static dispatch only: monomorphised, no `Box`, no vtable, no allocation. `Box<dyn Command>` and any registry or `HashMap` dispatch are **rejected** — the cost is real and the command set is closed.
  - The exhaustive `match` **remains**: it converts `Commands` into the concrete type, which is also where exhaustiveness is enforced.
  - **Revisit trigger**: if the command set ever becomes dynamic or plugin-loaded, the factory precondition appears and D8 reopens.
  - The trait is the repository`s first async-fn-in-trait style contract. There is no `async-trait` dependency and none is added.
  - **The trait must not be created before its first impl**: `wm-cli` is a binary crate, so a `pub trait` with no implementors and no callers is dead code and would break `no-warnings`. It landed with the first command in step 3.
  - Naming notes recorded: `std::process::Command` is used only inside the Init arm, fully qualified and never imported, so it cannot collide with the trait; that must be preserved. The trait is implemented by Action enums and args structs, so `impl Command for SearchAction` reads slightly off, since `SearchAction` is a sub-action rather than the command itself; `Action` and `Runnable` were considered and rejected.

  ## Size budget and enforcement

  **Hard cap 300 lines per `.rs` file; `main.rs` at most 80; functions at most 20 (named exceptions above).**

  Enforced by a deterministic CI check in the `check` job of `.github/workflows/ci.yml`, placed after the existing `Build wm-cli` step, following the existing `#[allow(...)]`-ban precedent. Dependency-free `find`/`wc` shell, no new test directory.

  ## Explicit non-decisions

  | # | Topic | This decision states |
  |---|---|---|
  | D2 | clap surface mechanism | types placed in `shared/models/` with derives intact; mechanism decided later |
  | D3 | `AGENTS.md` template | travels inline with the Init arm; relocation decided later |
  | D4 | `EngineHandle` and `OnceLock` lifecycle | located in `shared/services/engine_proxy.rs`; **must reuse or extend** `apps/wm-core/src/engine/main_engine_factory.rs` — no second construction path. Internals open |
  | D5 | agent-sync cluster boundary | two shared service files today; finer split later |
  | D6 | output layer for 229 `println!` | per-command `print_*` functions; abstraction later |
  | D7 | `anyhow` versus typed errors | `Result<_, anyhow::Error>` preserved verbatim |
  | - | per-command construction | a `*_builder_service.rs` inside the command directory, never a central factory |
  | - | `traits/` role-directory gap | flagged; recommendation is to amend CONVENTIONS to add `traits/`, or document `shared/traits/` as the accepted exception |

  ## Migration ordering

  Invariant for every commit: `cargo check --workspace` clean, `cargo clippy -p wm-cli -- -D warnings` clean, suites green. Every extraction commit follows a two-phase discipline: move verbatim, verify byte-identity, then decompose, then re-run goldens.

  | Step | Content |
  |---|---|
  | 0 | **Characterization net** (DONE): golden `--help` for the binary and all 21 subcommands plus per-leaf stdout goldens, generated from the pre-move binary. 33 goldens. The harness now fails loudly if the binary is stale. |
  | 1a | **Move the 16 clap surface types verbatim** into `shared/models/` (DONE). |
  | 1b | **Per-leaf args structs and tuple variants**, each inside that command`s own step-3 commit (DONE for all leaves). |
  | 2 | `shared/helpers/` and the shared services (DONE). |
  | 3 | **21 per-command commits** — DONE for 16: version, migrate_memory, agents, validate, decide, config, model, time, log, lint, task, source, index, search, status, setup, page, graph, mcp. REMANNING: health, init. |
  | 4 | Extract `dispatch` to `services/command_dispatch_service.rs`, slim `main.rs`, add the CI budget step (red to green in the same commit), add in-process routing tests. |
  | 5 | Optional and separate: re-home `mcp_server.rs`; remove the unused `rayon` and `chrono` dependencies. |

  **Riskiest step is 3**, because decomposition means arms are no longer moved verbatim and byte-identity `git diff` no longer covers the change. The step-0 goldens carry that load.

  ## Risks

  | Risk | Consequence | Mitigation |
  |---|---|---|
  | Decomposition drifts stdout or exit codes | behaviour freeze broken | goldens per leaf, two-phase commits, bottom-up extraction only |
  | A cap-driven idiom change sneaks in | new panic paths or worse style in a frozen refactor | R8: the cap never justifies a worse idiom; exemptions instead |
  | clap `Args` flattening into a tuple variant does not work | leaf args structs unusable | PROVEN in batch 1 — accepted, byte-identical help |
  | `unused_async` on non-awaiting impls | warning breaks `no-warnings` | RPITIT signature; and it does not fire on trait-impl methods |

  ## Reviewer amendments incorporated

  | Wave | Amendment |
  |---|---|
  | R1 | Uniform one directory per command (all 21); no inline arms; Health decomposed inside its own directory; budget as a CI check |
  | R2 | Shared module first-class |
  | R3 | Share means 2 or more consumers, counted mechanically; decomposition mandate: 20-line functions, names carry intent, no comments needed |
  | R4 | Dispatch is an extracted exhaustive match, not a factory; D4 reuses `main_engine_factory.rs` |
  | R5 | The `Command` trait is mandatory and shared at `shared/traits/command.rs`, unsuffixed, mirroring wm-core, with impls in each command module |
  | R6 | Trait signature restored to RPITIT with an explicit `+ Send`; **21/21 coverage** through per-leaf args structs; `shared/helpers/` populated as a recorded exception; attribution corrected |
  | R7 | **Both** helpers promoted into `shared/helpers/`; step 1 split into 1a and 1b |
  | R8 | Per-command flat dispatchers (`run_page_action`, `run_graph_action`) are **exempt** from the 20-line cap, and the 14 `unreachable!()` sites introduced to satisfy the cap are **rejected and removed**. Binding rule recorded: the cap must never drive a worse idiom |

  ## Open recommendations

  1. Create `@wiki/rules/file-size-budget` (one invariant plus a pointer to the CI step). Requires reviewer approval because creating rules is a governance act.
  2. Land the unused `rayon` and `chrono` dependency removal as a **separate** commit.
  3. Amend CONVENTIONS to add `traits/` to the role-directory list, or document `shared/traits/` as the accepted exception.
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
