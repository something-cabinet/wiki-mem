---
title: wm_code read policy should follow .gitignore, not a hardcoded hidden-file ban
type: task
id: "wiki:tasks:wmcode-read-policy-should-follow-gitignore-not-a-hardcoded-hidden-file-ban"
status: done
priority: medium
tags: [code-intel, security, policy, wm-tool-reliability]
acceptance_criteria:
  - text: "wm_code.file, wm_code.search, wm_code.symbols and wm_code.deps resolve readability from the repository's ignore rules, with .github/workflows/** and .wm/wiki/** reachable and target/, node_modules/, .idea/, .vscode/, .claude/ still excluded"
  - text: "Root confinement is unchanged: a path resolving outside the project root is still rejected and audited, and there is a test for it"
  - text: ".git/ is hard-excluded by an explicit test, independent of ignore rules"
  - text: ".env and .env.* are refused, pinned by a test"
  - text: "The four divergent skip lists are consolidated into one policy layer, with any remaining explicit exclusion justified in code-free terms (a recorded reason, no comments)"
  - text: "The dependency choice (ignore crate vs globset vs hand-rolled) is recorded as a decision with its cost"
  - text: "Existing code-intel, CLI and wm-core suites stay green; no comments, no #[allow], functions at most 20 lines"
implementation_notes: |2-

  ## Emphasis from the reviewer (2026-10-15): nothing inside .gitignore may be INDEXED

  Readability is the smaller half. The binding requirement is that the **code index** contains no gitignored file — the read paths, the watcher and the code-index walk must all share one ignore policy.

  **Current index state (measured via sqlite3 on .wm/state/code.db):** 385 files, 3,333 symbols, 19,441 raw edges, 7,374 resolved edges. A direct query for indexed paths under `.kiro`, `.slim`, `tmp/`, `.knowns`, `*/.wm/`, `.claude`, `.vscode`, `.idea`, `node_modules/`, `target/` returns **empty**.

  **But that is luck, not enforcement.** `.kiro` and `.slim/deepwork` both exist and are both gitignored, and neither appears in any skip list — they are excluded only because they hold no files whose extension the index accepts (rs/ts/tsx/py/go/html/svelte). The static case is unambiguous: `ingest_service` skips only `SKIP_DIRS_CODE` (`.git`, `node_modules`, `target`) plus `.claude/.opencode/.vscode/.idea`; `.gitignore:43` ignores `tmp/`; `.rs` is accepted. A gitignored `tmp/probe/probe.rs` would therefore be indexed.

  **Live verification was attempted and BLOCKED, not passed.** Creating `tmp/probe/probe.rs` (confirmed ignored by `git check-ignore -v`) and running `wm-cli index code` failed with a DB lock error — see `@wiki/tasks/wiki-tool-reliability-wm-index-code-cannot-run-while-an-mcp-server-is-open-codedb-lock`. So no live proof of the violation exists yet, and this task must add the test that pins it.

  **Additional requirement for this task:** a test that creates a source file inside a gitignored directory and asserts it is absent from `code_files`, from `code_symbols` and from the resolved edges. The policy must be enforced by rule evaluation, not by an extension allowlist happening to exclude today`s gitignored content.
  Owner decision 2026-10-16 (EXTRACT-03 review): APPROVED that wm_code.search/symbols/deps prune .wm and wm_code.file denies .wm reads.
  This is a consequence of wm-tools delegating walking to wm-code-intel's ignore policy (D-6 step 2, AC-7) rather than carrying its own traversal. Recorded here per D-8, which assigns the WIKI_DIR/read-policy semantic bug its own task.
  Rationale accepted: wiki reads are owned by wm_page.*, and excluding .wm is appropriate for a source-code tool. The underlying hardcoded hidden-file ban should still be resolved per @wiki/decisions/code-read-policy-follows-gitignore.

  ## Emphasis from the reviewer (2026-10-15): nothing inside .gitignore may be INDEXED

  Readability is the smaller half. The binding requirement is that the **code index** contains no gitignored file — the read paths, the watcher and the code-index walk must all share one ignore policy.

  **Current index state (measured via sqlite3 on .wm/state/code.db):** 385 files, 3,333 symbols, 19,441 raw edges, 7,374 resolved edges. A direct query for indexed paths under `.kiro`, `.slim`, `tmp/`, `.knowns`, `*/.wm/`, `.claude`, `.vscode`, `.idea`, `node_modules/`, `target/` returns **empty**.

  **But that is luck, not enforcement.** `.kiro` and `.slim/deepwork` both exist and are both gitignored, and neither appears in any skip list — they are excluded only because they hold no files whose extension the index accepts (rs/ts/tsx/py/go/html/svelte). The static case is unambiguous: `ingest_service` skips only `SKIP_DIRS_CODE` (`.git`, `node_modules`, `target`) plus `.claude/.opencode/.vscode/.idea`; `.gitignore:43` ignores `tmp/`; `.rs` is accepted. A gitignored `tmp/probe/probe.rs` would therefore be indexed.

  **Live verification was attempted and BLOCKED, not passed.** Creating `tmp/probe/probe.rs` (confirmed ignored by `git check-ignore -v`) and running `wm-cli index code` failed with a DB lock error — see `@wiki/tasks/wiki-tool-reliability-wm-index-code-cannot-run-while-an-mcp-server-is-open-codedb-lock`. So no live proof of the violation exists yet, and this task must add the test that pins it.

  **Additional requirement for this task:** a test that creates a source file inside a gitignored directory and asserts it is absent from `code_files`, from `code_symbols` and from the resolved edges. The policy must be enforced by rule evaluation, not by an extension allowlist happening to exclude today`s gitignored content.
---

**Directive (reviewer):** `wm_code.*` should decide readability from the repository's ignore rules, not from a blanket ban on hidden path components.

**Why:** the ban produces wrong answers for tracked content while the repo already curates the intent in `.gitignore`.

**Verified current state:**
- `wm_code.file` refuses any path containing a `.`-prefixed component (`apps/wm-core/src/mcp/tools/code.rs:886-888`).
- The same idea also lives inside `confine_strict` via `ERR_HIDDEN` (`apps/wm-core/src/shared/helpers/path_confine_helper.rs:108`), so the ban is part of the security-adjacent confinement path (WM-003/WM-004 hardening), not merely a code-tool quirk. The watcher and `wm-code-intel` carry two more copies.
- **Four divergent skip lists express the same intent:** `wm-constants::SKIP_DIRS` = `.wm .agent .agents .git .github`; `wm-constants::SKIP_DIRS_CODE` = `.git node_modules target`; `code.rs::SKIP_DIRS` = the 11-entry union re-declared locally; `wm-code-intel ingest_service::SKIP_DIRS` = `.claude .opencode .vscode .idea`.
- `.gitignore` already covers the same ground: `target/`, `node_modules/`, `.idea/`, `.vscode/`, `.claude/`, `.env`, `.env.*`, `**/.wm/skills/`, `**/.wm/log.jsonl`, `tmp/`, `.slim/deepwork/`.
- No `ignore` or `globset` dependency exists; only `walkdir`.

**Concrete wrongness this fixes:**
- `.github/workflows/**` is tracked and not ignored, yet it is unreachable — `.github` is in `SKIP_DIRS` and the dir name is hidden. The repo's own CI configuration cannot be searched.
- `.wm/wiki/**` is tracked content and is currently refused (the error observed while reviewing code reads).

**Invariants that must NOT regress (these are the security controls):**
1. **Root confinement stays.** `confine` (lexical normalisation plus `starts_with(root)` plus the audit sink) remains mandatory. An ignore policy is an *additional* filter, never a replacement.
2. **`.git/` stays hard-excluded regardless of ignore rules.** Git does not ignore its own `.git`, and this repo's `.git/config` held a live PAT until 2026-10-15. A pure gitignore policy would expose it — a regression.
3. **Secrets stay protected.** `.env` and `.env.*` are gitignored, so an ignore-based policy covers them, but this must be pinned by a test rather than assumed.
4. `target/` and `node_modules/` behaviour is unchanged (they are ignored).

**Design direction:** adopt the `ignore` crate (ripgrep's) rather than hand-rolling gitignore semantics — it handles nested `.gitignore` files, `.git/info/exclude`, global excludes, negation, anchoring, directory-only patterns, and never descends into `.git`. Replace the four skip lists with one policy layer, keeping explicit functional exclusions only where they are genuinely not ignore-based (for example `.wm` for the wiki scan, because the wiki has its own index).

**Note:** this changes a security-adjacent control and adds a dependency, so it needs a decision before implementation.