---
id: wiki:tasks:cli
title: 'Wiki Tool Reliability: wm-cli page — stdin-only content contract'
type: task
status: done
tags:
- bug
- cli
- tool-reliability
relates_to:
- type: references
  target: wiki:rules:tool-reliability-bug-tracking
acceptance_criteria:
- text: wm-cli gains a page update subcommand equivalent to wm_page.update
- text: wm-cli page create reads page content from stdin only — there is NO --content flag
- text: wm-cli page update reads a JSON update payload from stdin only — there is NO --content flag
- text: Passing --content to page create/update is rejected by clap (regression test guards this)
---

**Severity:** Medium

**Decision (oracle D3, ratified):** `wm-cli page create` and `wm-cli page update` read their content from **stdin only**. A `--content` flag was proposed to fix multiline/frontmatter input but was **rejected**; the blessed contract is the pipe idiom.

**Contract:**

1. **`page create` reads content from stdin** — the entire stdin stream becomes the page body (frontmatter is generated from `--page-type`, title, and path):
   ```
   echo '# Hello' | wm-cli page create concepts/hello "Hello"
   ```
   Multiline content and YAML frontmatter pass through stdin untouched — no clap "unexpected argument" errors.

2. **`page update` reads a JSON update payload from stdin** (equivalent to `wm_page.update`):
   ```
   echo '{"title": "New Title", "status": "in-progress"}' | wm-cli page update wiki:concepts:hello
   ```

3. **There is no `--content` flag** anywhere in `page create` / `page update`. Passing it is rejected by clap (`error: unexpected argument`), guarded by the `test_regression_content_flag_rejected` regression test.

**Reproduction (stdin contract):**
1. Run: `echo 'Body with --- frontmatter-like text' | wm-cli page create "test/foo" "Test"`
2. Observe: page is created with the piped body; the `---` never reaches clap's argument parser.
3. Run: `echo '{"title": "Renamed"}' | wm-cli page update wiki:tasks:test:foo`
4. Observe: page title updates via stdin JSON.