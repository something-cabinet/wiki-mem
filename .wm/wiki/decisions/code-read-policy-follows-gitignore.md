---
title: Code Read Policy Follows .gitignore (ignore crate, single policy layer)
type: decision
id: "wiki:decisions:code-read-policy-follows-gitignore"
status: approved
tags: [code-intel, security, policy, decision]
---

schema_version: 1
state: |-
  ## Context

  `wm_code.*` decided readability from a **blanket ban on hidden path components**, and the same intent was expressed in **four divergent skip lists** plus a copy inside the confinement helper. The result was wrong answers for content that git tracks.

  | Site | Behaviour |
  |---|---|
  | `apps/wm-core/src/mcp/tools/code.rs:886-888` | `wm_code.file` refuses any path with a `.`-prefixed component |
  | `apps/wm-core/src/shared/helpers/path_confine_helper.rs:108` | `confine_strict` rejects hidden components (`ERR_HIDDEN`) — inside the security-adjacent confinement path (WM-003/WM-004 wave) |
  | `apps/wm-core/src/engine/main_engine_factory.rs:141` | the watcher skips hidden directories |
  | `packages/wm-code-intel/src/services/engine_service.rs:751,772` | two further hidden checks |

  | Skip list | Entries |
  |---|---|
  | `wm-constants::SKIP_DIRS` | `.wm .agent .agents .git .github` |
  | `wm-constants::SKIP_DIRS_CODE` | `.git node_modules target` |
  | `code.rs::SKIP_DIRS` | the 11-entry union, re-declared locally rather than composed |
  | `wm-code-intel ingest_service::SKIP_DIRS` | `.claude .opencode .vscode .idea` |

  Meanwhile `.gitignore` already curates the same intent: `target/`, `node_modules/`, `.idea/`, `.vscode/`, `.claude/`, `.env`, `.env.*`, `**/.wm/skills/`, `**/.wm/log.jsonl`, `tmp/`, `.slim/deepwork/`. The hardcoded lists therefore approximate a source of truth that already exists, and disagree with each other while doing it.

  **Concrete wrongness:** `.github/workflows/**` is tracked and not ignored, yet unreachable — `.github` sits in `SKIP_DIRS` and the directory name is hidden, so the repository cannot search its own CI configuration. `.wm/wiki/**` is tracked content and is refused outright.

  ## Decision

  **Readability is decided by the repository`s ignore rules, not by path-component names.** Specifically:

  1. Adopt the **`ignore` crate** (ripgrep`s) as the single ignore-rule evaluator. It handles nested `.gitignore` files, `.git/info/exclude`, global excludes, negation, anchoring, directory-only patterns, and it never descends into `.git`.
  2. **Consolidate the four skip lists and the scattered hidden checks into one policy layer.** An entry survives only when it is genuinely not ignore-based, and the reason must be recorded rather than implied — for example `.wm` for the wiki scan, whose content has its own index.
  3. Apply it to all four `wm_code.*` read paths, the watcher, and the code-index walk, so one policy governs indexing and reading alike.

  ## Invariants that must not regress

  These are the security controls, and they are not negotiable by this decision:

  1. **Root confinement stays.** `confine` — lexical normalisation, `starts_with(root)`, and the audit sink — remains mandatory. The ignore policy is an *additional* filter, never a replacement for confinement.
  2. **`.git/` stays hard-excluded, independent of ignore rules.** Git does not ignore its own `.git`, and this repository`s `.git/config` held a live PAT until 2026-10-15. A pure gitignore policy would expose it; the `ignore` crate excludes `.git` by default, which is part of why it is the chosen tool.
  3. **Secrets stay protected.** `.env` and `.env.*` are gitignored, so the policy covers them — but this must be pinned by a test rather than assumed.
  4. `target/` and `node_modules/` behaviour is unchanged; they are ignored already.

  ## Options considered

  | Option | Verdict |
  |---|---|
  | **`ignore` crate plus full consolidation** | **Chosen.** One dependency, one policy, and gitignore semantics handled by the tool the ecosystem uses. |
  | `ignore` crate but scoped to `wm_code.*` only | Rejected: leaves the four lists divergent and `.github` still absent from the code index. |
  | Hand-rolled `.gitignore` parsing with the existing `walkdir` | Rejected: precedence, negation, anchoring and directory-only patterns are subtly wrong in naive implementations, and a wrong filter silently hides or exposes files. |
  | Targeted un-ban of `.github` and `.wm` only | Rejected: fixes the symptoms while leaving four disagreeing lists as the policy. |

  ## Consequences

  - **A new dependency** enters the workspace (`ignore`), and it is the only dependency whose sole job is policy evaluation.
  - **The code index grows**: `.github/workflows/**` becomes indexable, so symbol, dependency and edge counts change. Any baseline numbers recorded for the code index must be re-taken.
  - **`.wm/wiki/**` becomes readable by `wm_code.file`**, which is intended — it is tracked content.
  - **Four lists and several hidden checks are removed**, so the codebase loses duplication and gains a single place to reason about readability.

  ## Implementation ordering

  | Step | Content |
  |---|---|
  1 | Add the policy layer with the `ignore` crate, plus tests for the four invariants (confinement, `.git`, `.env`, `target`/`node_modules`), before wiring any caller |
  2 | Point the `wm_code.*` read paths at it and delete the local hidden check in `code.rs` |
  3 | Point the watcher and the code-index walk at it; delete the ingest and `code.rs` skip lists |
  4 | Decide `.wm` explicitly for the wiki scan (its content has its own index) and record the reason |
  5 | Re-take any code-index baseline numbers, since `.github` becomes indexed |

  ## Risks

  | Risk | Consequence | Mitigation |
  |---|---|---|
  | The policy layer replaces confinement by accident | a path outside the root becomes readable | invariant 1 plus an explicit rejection test |
  | `.git` exposed through a relaxed rule | credential and history exposure | invariant 2 plus a test that does not depend on ignore rules |
  | Secrets become readable because `.env` is not ignored in some other checkout | leaks | invariant 3 pinned by a test |
  | Index growth from `.github` surprises a baseline | misleading before/after comparison | re-take baselines in step 5 |

  ## Related

  - `@wiki/tasks/wmcode-read-policy-should-follow-gitignore-not-a-hardcoded-hidden-file-ban` — the implementation task
  - `@wiki/tasks/path-confinement-helper-and-userpath-newtype` — the confinement control this must not weaken
  - `@wiki/tasks/wm003-page-doc-path-traversal` and `@wiki/tasks/wm004-source-arbitrary-file-read` — the hardening wave that introduced the hidden-component ban
  - `@wiki/tasks/wire-audit-sink-for-security-rejections` — the audit sink that must keep firing on escapes
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
  reversibility: true
  confidence: high
  impact: component
