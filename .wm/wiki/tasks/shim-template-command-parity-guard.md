---
title: Add a guard test that fails when a shim template names a non-existent CLI command
type: task
id: "wiki:tasks:shim-template-command-parity-guard"
status: todo
tags: [testing, agent-guidance, ci]
implementation_notes: 'Deferred: implemented + mutation-verified 2026-10-15, then reverted to stop sharing a build target with the concurrent wm-cli main.rs rewrite. Re-add when wm-cli compiles. Inline-span coverage hole documented in the body.'
---

Severity: Medium

Implement @wiki/specs/shim-template-command-parity-guard — the test that fails when an embedded agent-handbook template names a CLI command that does not exist.

Why now: this is the check that was missing when @wiki/specs/remove-self-install-flow closed with its doc-update ACs unmet, leaving six templates shipping `wm-cli serve`, `wm upgrade` and `wm init --full` to every new project.

## Acceptance Criteria

- [ ] Test loads the shims from the embedded file set, not from disk
- [ ] Extracted subcommands are resolved against clap's live subcommand tree (one level deep)
- [ ] Bare `wm` invocations fail the test
- [ ] Failure output names the template file and the offending line
- [ ] **Inline backtick spans are scanned too, not just fenced bash blocks** (added 2026-10-15 — see Status)
- [ ] Checked-in root handbooks are asserted byte-identical to their templates
- [ ] Mutation check: injecting `wm-cli serve` makes the test fail
- [ ] Mutation check: an inline `wm-cli bogus` makes the test fail
- [ ] `cargo test` clean, zero warnings, no new dependencies

## Files

- `apps/wm-cli/src/` — test module (see spec D1)
- `apps/wm-core/src/embed_files/shims/` — read-only input to the test
- root handbooks — read-only input to the parity assertion

## Depends On

@wiki/tasks/agent-handbooks-reference-nonexistent-cli-commands — the template content must be corrected for AC-2 to pass.

## Status 2026-10-15 — implemented, verified, then reverted (deferred)

The guard was written as `apps/wm-cli/src/shim_command_parity_guard.rs` plus one `mod` line in `main.rs`, and it worked: `cargo test -p wm-cli` passed (2 tests, zero warnings), and injecting `wm-cli serve` produced `shims/copilot-instructions.md:42: wm-cli serve ...` and failed the run.

Reverted on request: the crate is currently non-compiling because of an unrelated concurrent workstream (`apps/wm-cli/src/services/` and `apps/wm-cli/src/shared/` are new and untracked; `main.rs` lost ~1969 lines). Two workstreams should not share a build target, and the guard's `use crate::shared::models::Cli;` binds to that same in-flight module layout. Re-add once that refactor settles.

## Coverage hole to fix when re-adding

The reverted guard scanned **only fenced ```bash blocks**. Proven by mutation: changing an inline span to `\`wm-cli bogus\`` passed the test. This matters because the original defect included an inline invocation (`Run \`wm mcp\` to start the MCP server`), so the guard as written would not have caught one of the two real regressions it exists for.

The fix (written, then reverted with the rest): also scan inline backtick spans on non-fenced lines, accepting a span only when its first token is exactly `wm-cli` or `wm` — so `wm_page.get`, `wm_search.query`, `wm_init`, `wm_*` and `wm-init` are ignored — stripping an optional `$ ` prefix and reusing the same subcommand resolution.

## Required verification on re-add

1. Green on the corrected templates with no false positives from `wm_*` tool names.
2. Fenced mutation (`wm-cli serve`) fails, naming file and line.
3. Inline mutation (`wm-cli bogus`) fails — the check that was missed.
4. Parity mutation (edit a root handbook only) fails.
5. Zero warnings, no new dependencies, no `#[allow]`.