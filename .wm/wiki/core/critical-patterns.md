---
title: Critical Patterns
type: core
tags: [critical]
status: active
---

schema_version: 1
state: |-
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

  ## 2026-08-12 tokio::spawn from a std thread panics (\"no reactor running\")

  **Category:** pattern / failure
  **Source:** @task-test-suite-simplification
  **Tags:** [tokio, concurrency, watcher, testing]

  Calling `tokio::spawn` from a plain `std::thread` panics and kills that thread. The notify watcher thread in `MainEngine::with_root` is a std thread, so `notify_file_changed` (lsp feature) died on the first external file change — silently, after the graph update had already applied. Only a watcher-level test (write to disk, poll with deadline) caught it; direct `handle_file_change` tests never do. Fix: `Handle::try_current()` → `tokio::spawn`, else a one-shot current-thread runtime in a std thread. **Test the real thread, not the handler.**

  **Full entry:** @wiki/memory/watcher-thread-tokio-spawn-panic

  ## 2026-08-12 Specialist result transmission can silently fail

  **Category:** pattern / failure
  **Source:** @task-cli-mcp-in-process-refactor
  **Tags:** [orchestration, subagents, workflow]

  Oracle/fixer sessions can complete their work (context-read lists prove the reads) while the final result message returns EMPTY — 6 consecutive empty oracle transmissions across two sessions in one day. Don't burn repeated identical reissues; verify the tree (git status/diff) to distinguish \"landed work, lost message\" from \"wrote nothing\", run the review checklist yourself when the reviewer's delivery is dead, and keep the deepwork file as the durable gate record. Sessions become resumable once the board reconciles them.

  **Full entry:** @wiki/memory/agent-empty-result-transmission

  ## 2026-08-14 Review gates must return non-empty verdicts

  **Category:** pattern / failure
  **Source:** @task-re-run-wave-1-review-gate-t1t4t5--initial-gate-returned-empty
  **Tags:** [orchestration, review, wm-flow, workflow]

  A review-gate task that returns an EMPTY result is NOT a review — lifecycle \"completed\" only means the session ended. The Wave 1 gate (ora-1) returned nothing and the wave could have proceeded to Wave 2 on unreviewed code. **Validate review-gate output before accepting it: empty/no-verdicts → treat as not-reviewed, re-dispatch with an explicit \"return GO / GO-with-findings / NO-GO per lane\" requirement.** Prefer per-lane gates over one mega-gate covering many disjoint lanes (partial/empty results are easier to detect).

  **Full entry:** @wiki/concepts/empty-review-gate-result

  ## 2026-08-14 Measurement-gated deletion for multi-path scoring code

  **Category:** pattern / failure
  **Source:** @task-add-golden-query-eval-harness-then-reduce-ranking-tiers
  **Tags:** [search, ranking, measurement, testing]

  Deleting \"redundant\" scoring boosts (+8/+4/+2/+7/+3) regressed keyword-only exact-match ranking. The boosts were inert for the HYBRID path (RRF consumes only rank order) but ACTIVE for the keyword-only path (normalized BM25 scores consumed by value; `post_rrf_rerank` runs only on the hybrid+embedder branch). A 50-query golden eval reported recall@5=1.0 before/after — saturated, all multi-word queries — so it couldn't detect the exact-match/exact-id tie-break regression. **Before deleting any scoring code: (1) build an eval exercising EACH search path (keyword, hybrid, semantic), (2) include exact-match/exact-id/tag queries not just natural-language, (3) measure recall@k per path before/after EACH deletion, (4) revert immediately if any path regresses.** Deletion was reverted; consolidation deferred behind exact-match golden queries.

  **Full entry:** @wiki/patterns/measurement-gated-deletion

  ---

  ## 2026-10-13 A tool success payload is not proof of a durable write

  **Category:** failure / pattern
  **Source:** @task-wmtask-checkac-and-status-updates-report-success-without-persisting
  **Tags:** [tool-reliability, persistence, yaml, testing]

  `wm_task check_ac` returned `{"checked":[0]}` on every call yet wrote zero `checked: true` to any page — an `index == 0` no-op guard plus a `flush_pending` that set `inserted = true` without pushing (the first list item flushed before the target indent was known), so no index ever wrote. A false-positive test (`contains("checked: true")` against a fixture that already contained it) hid it. **Verify the artifact, not the response: after any state-changing write, re-read the file from disk and assert the specific field changed; return an error otherwise. Never assert on pre-existing fixture content. Order derived stores (version history, indexes) after the source-of-truth write.**

  **Full entry:** @wiki/patterns/verify-writes-by-read-back

  ---

  ## 2026-10-15 A tool error is not proof of a failed write — and concurrent same-page writes silently lose updates

  **Category:** failure / pattern
  **Source:** @wiki/tasks:wiki-tool-reliability-wmpageupdate-returns-ioerror-on-success
  **Tags:** [tool-reliability, persistence, concurrency, honesty]

  `wm_page.update` returned `IO_ERROR` on a write that **fully landed** — twice, verified by reading the file back. Worse, two updates issued **concurrently against the same page** produced the full failure set at once: one call reported the false error while its change applied, and the other reported **success while silently discarding its write** (the appended note vanished; the file shrank 73 bytes). Re-issuing sequentially worked. So a tool response is unreliable in *both* directions now, and the false error is arguably worse for agents than the false success already recorded — the agent either duplicates content on retry or reports failure for completed work. **Read the artifact back after every state-changing write, and never issue two writes to the same page concurrently.**

  **Full entry:** @wiki/patterns/verify-writes-by-read-back

  ---

  ## 2026-10-15 `extra_frontmatter` is silently discarded whenever `content` is supplied

  **Category:** failure
  **Source:** @wiki/tasks:wiki-tool-reliability-wmpageupdate-ignores-extrafrontmatter-with-content
  **Tags:** [tool-reliability, record-format, mcp, api]

  Both `wm_page.create` and `wm_page.update` ignore `extra_frontmatter` when `content` is supplied in the same call — no error, no warning. Consequence: **acceptance criteria and priority cannot be set at creation time through any WM tool path**; every task needs a create-then-rewrite pair, which is exactly the two-call shape that raises exposure to the concurrent-write lost-update defect above. Same class as the issue-#126 `r#type` field: a declared input the handler never consumes, shipping silently. **If the schema accepts it, the handler must consume it — wire it or delete it.**

  **Full entry:** @wiki/tasks:wiki-tool-reliability-wmpageupdate-ignores-extrafrontmatter-with-content

  ## 2026-10-15 Do not decide the owner's preferences for them

  **Category:** failure / process
  **Source:** @wiki/concepts:failure-agent-decided-owner-preferences
  **Tags:** [authority, process, curation]

  Asked for a way to share rules across projects, an agent produced a curated global rule set - deciding which of 46 rule pages go global, which retire, and which variant is canonical. All of that is the owner's preference, not a technical convergence problem with a best answer. The owner's reply: "global rule should be for user to decide." The agent had already written "Curation is human, tooling only reports" into its own spec and then did the work anyway - **writing a rule is not the same as obeying it**. When a deliverable encodes the owner's taste or authority, emit options with a labelled recommendation and blank decision fields; never resolutions. Cost: four review rounds, four specs and thirteen tasks, reversed in one message.

  **Full entry:** @wiki/concepts/failure-agent-decided-owner-preferences

  ---

  ## 2026-10-16 A shared working tree is not a lock

  **Category:** failure / process
  **Source:** @wiki/tasks/extract-wm-core-01-finalize-step-0
  **Tags:** [orchestration, git, worktree, concurrency]

  Two agent sessions in one checkout: one session's subagent ran `git stash -u` then reset, to get a clean test baseline, and silently swept the other session's uncommitted edits, untracked pages, and a just-approved spec record. Recover tracked files from `stash@{0}` and untracked from `stash@{0}^3`; copy them outside the repo immediately. **Give each concurrent session its own `git worktree`, and check `git reflog` / `stash list` before assuming a change is yours.**

  **Full entry:** @wiki/concepts/concurrent-agent-shared-working-tree-hazard
questions:
  - id: kind
    type: choice
    instructions: What kind of core document is this?
    options:
    - conventions
    - architecture
    - patterns
    - glossary
    - reference
  - id: audience
    type: choice
    instructions: Who is this core document written for?
    options:
    - human
    - agent
    - both
  - id: mutability
    type: choice
    instructions: How mutable is this core document?
    options:
    - stable
    - evolving
    - frozen
answers: {}
