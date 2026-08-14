---
title: Critical Patterns
type: core
tags: [critical]
status: active
---

---
title: Critical Patterns
type: core
tags: [critical]
status: active
---

---

## 2026-08-10 Frontmatter corruption — never whole-block YAML round-trips

**Category:** pattern / failure
**Source:** @task-wm-task-update-frontmatter-corruption
**Tags:** [yaml, frontmatter, corruption, wiki]

Editing YAML frontmatter by parsing the whole block with serde_yaml and re-serializing corrupts data: unquoted ids like `652e07` become floats (`6520000000.0`), unmodeled fields get dropped, and empty maps emit `{}` blocks. **Always edit frontmatter line-based** (`set_yaml_field`/`remove_yaml_block`/`ac_set_checked` in yaml_helper.rs) and **always double-quote `id`**. Validator rules catch sci-notation ids, duplicate blocks, and id mismatch. A real task file was corrupted this way before the fix; 33 wiki files were repaired.

**Full entry:** @wiki/patterns/line-based-frontmatter-editing

## 2026-08-14 Dead input fields masked by #[allow(dead_code)]

**Category:** failure / decision
**Source:** @task-wmdoc-typetags-frontmatter-persistence (issue #126)
**Tags:** [lint, contract, mcp, api, frontmatter]

`wm_doc.create` declared `r#type` in its schema but the handler never wired it — the field was dead, and `#[allow(dead_code)]` silenced the compiler warning that would have caught it. The annotation's only function was hiding a contract defect; the bug shipped silently and broke typed pages until an index rebuild. **Never put `#[allow(dead_code)]` on an API input field: if the schema accepts it, the handler must consume it — wire it or delete it.** Clippy cannot enforce this (no attribute-ban lint exists), so it is enforced by a deterministic CI grep; the sanctioned replacement is `#[expect(dead_code)]`, which errors when the lint stops firing and can never mask a live field.

**Full entry:** @wiki/decisions/clippy-lint-curated-list-not-all

## 2026-08-10 MCP proxy architecture — privileged channel + token split

**Category:** decision
**Source:** @task-22ed6a
**Tags:** [mcp, proxy, architecture, security]

MCP is a thin stdio→HTTP proxy to the wm-server daemon targeting a privileged `POST /api/mcp/tools/{list,call}` channel with a **separate mcp-token** — the web-token surface stays read-only, so a browser-side token leak can never authorize writes. Dynamic `tools/list` from the registry (no STATIC_TOOLS array). Tool errors are HTTP 200 + `{success:false}` mapped to MCP `isError:true`; only auth/transport failures are non-200. ureq in `spawn_blocking`, token re-read + retry-once on 401.

**Full entry:** @wiki/decisions/mcp-proxy-privileged-channel-token-split

## 2026-08-10 Refresh derived state at the write path

**Category:** pattern
**Source:** @task-wm-task-update-frontmatter-corruption
**Tags:** [architecture, graph, cache, stale]

If a write path mutates state that reads consult (graph/index snapshot), refresh the derived snapshot **synchronously in the writer** — never rely on a file watcher to catch up. wm-server runs no watcher, so `wm_task.get` stayed stale until `wm_index_rebuild`; the fix was calling `graph::handle_file_change` after every page write.

**Full entry:** @wiki/patterns/refresh-derived-state-at-write-path

## 2026-08-10 Verify tree state before re-dispatching a failed lane

**Category:** pattern
**Source:** @task-c990b6
**Tags:** [orchestration, subagents, workflow]

A failed/cancelled subagent lane often still wrote complete code to the tree (this campaign: fix-7/8 CLI-over-HTTP, fix-14/16/19 core-leftovers all errored at the harness stage but had landed coherent work). Before re-dispatching, run `git status` + check the expected artifacts + `cargo check` — if the work is present and compiles, reconcile and verify instead of redoing. Saves minutes-to-hours and avoids conflicting edits on top of the partial run.

**Full entry:** @wiki/patterns/verify-tree-before-redispatching-failed-lane

## 2026-08-12 tokio::spawn from a std thread panics ("no reactor running")

**Category:** pattern / failure
**Source:** @task-test-suite-simplification
**Tags:** [tokio, concurrency, watcher, testing]

Calling `tokio::spawn` from a plain `std::thread` panics and kills that thread. The notify watcher thread in `MainEngine::with_root` is a std thread, so `notify_file_changed` (lsp feature) died on the first external file change — silently, after the graph update had already applied. Only a watcher-level test (write to disk, poll with deadline) caught it; direct `handle_file_change` tests never do. Fix: `Handle::try_current()` → `tokio::spawn`, else a one-shot current-thread runtime in a std thread. **Test the real thread, not the handler.**

**Full entry:** @wiki/memory/watcher-thread-tokio-spawn-panic

## 2026-08-12 Specialist result transmission can silently fail

**Category:** pattern / failure
**Source:** @task-cli-mcp-in-process-refactor
**Tags:** [orchestration, subagents, workflow]

Oracle/fixer sessions can complete their work (context-read lists prove the reads) while the final result message returns EMPTY — 6 consecutive empty oracle transmissions across two sessions in one day. Don't burn repeated identical reissues; verify the tree (git status/diff) to distinguish "landed work, lost message" from "wrote nothing", run the review checklist yourself when the reviewer's delivery is dead, and keep the deepwork file as the durable gate record. Sessions become resumable once the board reconciles them.

**Full entry:** @wiki/memory/agent-empty-result-transmission

---