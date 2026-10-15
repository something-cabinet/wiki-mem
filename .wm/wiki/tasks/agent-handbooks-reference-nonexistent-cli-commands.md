---
title: Agent handbook shim templates ship removed CLI commands to every project (wm-cli serve, wm upgrade, wm init --full)
type: task
id: "wiki:tasks:agent-handbooks-reference-nonexistent-cli-commands"
status: in-review
tags: [docs, agent-guidance, tool-reliability, embedded-templates]
---

Severity: High

## Root cause (found 2026-10-15)

The agent handbooks are **generated from templates embedded in the binary**, so the stale commands are not a one-off doc rot — they ship to every user's project.

Source templates: `apps/wm-core/src/embed_files/shims/` — **6 of the 7 templates still teach removed commands**:

| Template | Stale lines |
|---|---|
| `shims/AGENTS.md` | :33 `wm-cli serve`, :34-35 `wm init` / `wm init --full`, :36 `wm upgrade` |
| `shims/CLAUDE.md` | same block (:33-36) |
| `shims/GEMINI.md` | same block (:33-36) |
| `shims/REASONIX.md` | same block (:33-36) |
| `shims/copilot-instructions.md` | same block (:33-36) |
| `shims/OPENCODE.md` | :30 `wm-cli serve` |
| `shims/WIKI-MEM.md` | :60-62 `wm init` / `wm mcp` / `wm setup` (bare `wm`, wrong binary name); :54 `wm page migrate-records --only <path>` — missing `--apply` |

Those templates are copied verbatim into every project by `wm-cli init` / `wm-cli agents`, and the same text is checked into this repo (`AGENTS.md`, `OPENCODE.md`, `GEMINI.md`, `REASONIX.md`, `WIKI-MEM.md`).

## Why it exists

A **missed acceptance criterion of an already-completed spec**. `@wiki/specs/remove-self-install-flow` removed `wm upgrade` and `init --full` from the CLI and explicitly required:

- **FR-5** — "Update docs to reflect the removed flow: `WIKI-MEM.md` Quick Reference (remove `wm upgrade`, `wm init --full` lines)…"
- **AC-7 / AC-8** — "`WIKI-MEM.md` no longer documents `wm upgrade` or `wm init --full`"

The code half landed. The doc half was never done, and the check never ran.

## Two distinct defects

1. **Phantom commands** — `wm-cli serve`, `wm upgrade`, `wm init --full` do not exist. `Commands` has no `Serve`/`Upgrade` variant; `init` accepts only `--project`, `--platform`, `--no-wizard`.
2. **Wrong binary name** — the shims invoke bare `wm` (`wm init`, `wm search`, `wm lint`, `wm validate`, `wm task board`). The shipped binary is `wm-cli`; only clap's declared display name is `wm` (`apps/wm-cli/src/shared/models/cli_model.rs:6`), which is why the shims copied `wm`.

## Acceptance Criteria

- [x] No embedded shim template contains `wm-cli serve`, `wm upgrade`, or `init --full`
- [x] Every command in every shim template is verified against the clap `Commands` enum
- [x] One binary name (`wm-cli`) is used consistently across all shims
- [x] `WIKI-MEM.md` shim documents `wm-cli page migrate-records --apply`
- [x] The checked-in copies in this repo (`AGENTS.md`, `OPENCODE.md`, `GEMINI.md`, `REASONIX.md`, `WIKI-MEM.md`) match the regenerated output
- [ ] A test fails when a shim template references a subcommand that does not exist — **deferred** to @wiki/tasks/shim-template-command-parity-guard

## Files

- `apps/wm-core/src/embed_files/shims/*.md` (the fix belongs here first)
- `AGENTS.md`, `OPENCODE.md`, `GEMINI.md`, `REASONIX.md`, `WIKI-MEM.md` (regenerate)
- `apps/wm-core/src/embed_files/configs/` if any config template names a removed flag

## Status 2026-10-15 — content fixed, guard deferred

Fixed and verified:
- All 6 stale templates corrected. The Quick Reference now lists only live commands (`wm-cli init`, `mcp`, `setup <platform>`, `search query`, `task board`, `page list`, `lint check`, `validate`, `status`) with no bare `wm`.
- `shims/WIKI-MEM.md`: bare `wm` fixed at lines 54, 61-63 and 89; `migrate-records` now documented with `--apply`.
- Root handbooks re-synced and verified byte-identical to their templates: AGENTS.md, CLAUDE.md, GEMINI.md, OPENCODE.md, REASONIX.md, WIKI-MEM.md.
- Grep across all 13 handbooks: no phantom command, no bare `wm <command>` invocation.

Open items:
- The guard (last AC) is deferred — its implementation was reverted to avoid sharing a non-compiling build target with a concurrent `apps/wm-cli/src/main.rs` rewrite.
- `.github/copilot-instructions.md` was deliberately NOT synced: it is a different, older document (626 B vs the 2182 B template) with its own already-correct quick reference, and is not generated from the current shim. Decide separately whether to regenerate or delete it.
- The same class of rot may exist in `apps/wm-core/src/embed_files/skills/*` (10 skill files show as modified in the working tree). Not audited here.