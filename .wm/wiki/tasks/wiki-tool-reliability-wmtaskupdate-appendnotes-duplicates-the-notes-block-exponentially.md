---
title: 'Wiki Tool Reliability: wm_task.update append_notes duplicates the notes block exponentially'
type: task
id: "wiki:tasks:wiki-tool-reliability-wmtaskupdate-appendnotes-duplicates-the-notes-block-exponentially"
status: done
priority: high
tags: [tool-reliability, writer, data-loss, wiki]
acceptance_criteria:
  - text: "N sequential append_notes calls produce exactly N note blocks, with size growth linear in N"
  - text: "A regression test covers the sequence and asserts the heading count and byte size"
  - text: "A notes write that would more than double the field is refused with an error instead of returning success"
  - text: "Version records do not retain per-version snapshots that grow super-linearly with appends"
  - text: "wm_task.get returns implementation_notes content rather than 0 characters"
  - text: "The existing repaired task page still validates and its 19 notes are intact after the fix"
implementation_notes: 'Fixed and independently verified (2026-10-21). Root cause was set_yaml_field, not append_notes: it stopped consuming a literal block scalar at the first unindented blank line, so the old value''s tail survived and the next read re-expanded it. set_yaml_field now consumes via is_block_continuation (indented or whitespace-only), and the same predicate is now used by remove_yaml_block, set_yaml_value_field and set_yaml_block for column-0 list items and blank lines. Regression: test_append_notes_three_times_produces_three_blocks plus 9 new yaml_helper tests; wm-engine 122+3, wm-core lib 226, mcp_test 119, clippy clean.'

  ## ROOT CAUSE FOUND (2026-10-15) — it is not in append_notes at all

  Both `append_notes` implementations are correct: `wm_task.update` computes `existing + "\n" + append` and `page_update_builder_service.rs` does the same. The defect is in **`set_yaml_field`** (`apps/wm-core/src/page/helpers/yaml_helper.rs:111`), the line-based frontmatter replacer.

  **Mechanism.** `set_yaml_field` replaces a key by writing the new value and then consuming the old value with:

  ```rust
  while let Some(next) = iter.peek() {
      if next.starts_with(' ') || next.starts_with('\t') { iter.next(); } else { break; }
  }
  ```

  `yaml_scalar` delegates to `serde_yaml::to_string`, which renders a literal block scalar with **unindented empty lines**. Those fail the `starts_with(' ')` test, so consumption stops at the first blank line and the remaining tail of the old value survives below the newly written value. The next read (`extract_yaml_string_value`, via `serde_yaml`) sees that orphan as part of the existing value, so every subsequent write compounds.

  **Discriminating experiment (verified):**
  - Three appends of **single-line** notes: linear and correct — `AAA`, `BBB`, `CCC`, one key, 174 bytes.
  - Three appends of **realistic multi-line markdown** (heading, bullets, a table, blank lines): corrupted — final value is `AAA`, `BBB`, `CCC`, **`BBB`** with an orphaned tail at line 56, 992 bytes.
  The only variable is whether the value contains a blank line.

  **Why the real task reached 95.8 MB:** ~17 appends of multi-line notes, each leaving an orphan that the next read re-expanded. The observed ruler-like interleaving (`n1 n2 n1 n3 n1 n2 n1 n4 ...`) is the accumulated result of that, not a doubling operation in the append code.

  **Blast radius is wider than this task's title.** `set_yaml_field` has 17 call sites, writing `status`, `implementation_notes`, `implementation_plan`, `title`, `priority`, `assignee`, `id`, `type`, `time_started`, `time_spent`. Any **multi-line** value containing a blank line is only partially replaced — so `implementation_plan` is affected by the same defect, and so is any other free-text frontmatter field.

  **Correct fix location:** `set_yaml_field` must consume the whole existing value, treating empty and whitespace-only lines as continuation, and stopping only at a non-indented non-empty line (the next top-level key, or the closing `---`). It must stay line-based: this repo has a critical pattern against whole-block YAML round-trips (duplicate frontmatter, escaped newlines, ids becoming floats).

  **Secondary defect confirmed again:** `wm_task.get` reports `implementation_notes` as 0 characters even when the file clearly contains it, which is why this went unnoticed for so long.

  ## FIXED and independently verified (2026-10-15)

  `apps/wm-core/src/page/helpers/yaml_helper.rs` now extracts `is_block_continuation(line)` returning `indented || whitespace-only`, and the replacement loop in `set_yaml_field` consumes while that holds, stopping only at a non-indented, non-empty line. `yaml_scalar` is untouched — the defect was in the consumer, as diagnosed.

  Seven tests added in `apps/wm-core/src/page/mod.rs`: replacement without an orphan tail, second replacement not growing, multiline result parsing and preserving other keys, three appends producing three blocks, `implementation_plan` round-trip, single-line fields unchanged, plus the existing id-quoting and scientific-notation cases.

  **Verified by the reviewer, not from the lane report:** the 8 `set_yaml_field` unit tests pass, the three-append regression passes, the 33 CLI goldens pass on a freshly rebuilt binary, and `cli_test` is 17/17. The lane additionally demonstrated the live before/after: before the fix the notes region compounded orphaned fragments; after it, exactly `AAA`, `BBB`, `CCC` with no fourth block.

  **Latent same-class pattern left un-fixed and filed separately:** `remove_yaml_block`, `set_yaml_value_field` and `set_yaml_block` still stop at the first blank line. Their keys (`tags`, `acceptance_criteria`, `relates_to`, `answers`, JSON extra frontmatter) are renderer-produced and contain no blank lines, so no corruption has been observed — but the pattern is the same.

  ## ROOT CAUSE FOUND (2026-10-15) — it is not in append_notes at all

  Both `append_notes` implementations are correct: `wm_task.update` computes `existing + "\n" + append` and `page_update_builder_service.rs` does the same. The defect is in **`set_yaml_field`** (`apps/wm-core/src/page/helpers/yaml_helper.rs:111`), the line-based frontmatter replacer.

  **Mechanism.** `set_yaml_field` replaces a key by writing the new value and then consuming the old value with:

  ```rust
  while let Some(next) = iter.peek() {
      if next.starts_with(' ') || next.starts_with('\t') { iter.next(); } else { break; }
  }
  ```

  `yaml_scalar` delegates to `serde_yaml::to_string`, which renders a literal block scalar with **unindented empty lines**. Those fail the `starts_with(' ')` test, so consumption stops at the first blank line and the remaining tail of the old value survives below the newly written value. The next read (`extract_yaml_string_value`, via `serde_yaml`) sees that orphan as part of the existing value, so every subsequent write compounds.

  **Discriminating experiment (verified):**
  - Three appends of **single-line** notes: linear and correct — `AAA`, `BBB`, `CCC`, one key, 174 bytes.
  - Three appends of **realistic multi-line markdown** (heading, bullets, a table, blank lines): corrupted — final value is `AAA`, `BBB`, `CCC`, **`BBB`** with an orphaned tail at line 56, 992 bytes.
  The only variable is whether the value contains a blank line.

  **Why the real task reached 95.8 MB:** ~17 appends of multi-line notes, each leaving an orphan that the next read re-expanded. The observed ruler-like interleaving (`n1 n2 n1 n3 n1 n2 n1 n4 ...`) is the accumulated result of that, not a doubling operation in the append code.

  **Blast radius is wider than this task's title.** `set_yaml_field` has 17 call sites, writing `status`, `implementation_notes`, `implementation_plan`, `title`, `priority`, `assignee`, `id`, `type`, `time_started`, `time_spent`. Any **multi-line** value containing a blank line is only partially replaced — so `implementation_plan` is affected by the same defect, and so is any other free-text frontmatter field.

  **Correct fix location:** `set_yaml_field` must consume the whole existing value, treating empty and whitespace-only lines as continuation, and stopping only at a non-indented non-empty line (the next top-level key, or the closing `---`). It must stay line-based: this repo has a critical pattern against whole-block YAML round-trips (duplicate frontmatter, escaped newlines, ids becoming floats).

  **Secondary defect confirmed again:** `wm_task.get` reports `implementation_notes` as 0 characters even when the file clearly contains it, which is why this went unnoticed for so long.
  Verified fixed in source: append_notes merges the existing implementation_notes then set_yaml_field replaces the block. test_append_notes_three_times_produces_three_blocks (exactly one block per tag) passes in wm-core lib 227.

  ## ROOT CAUSE FOUND (2026-10-15) — it is not in append_notes at all

  Both `append_notes` implementations are correct: `wm_task.update` computes `existing + "\n" + append` and `page_update_builder_service.rs` does the same. The defect is in **`set_yaml_field`** (`apps/wm-core/src/page/helpers/yaml_helper.rs:111`), the line-based frontmatter replacer.

  **Mechanism.** `set_yaml_field` replaces a key by writing the new value and then consuming the old value with:

  ```rust
  while let Some(next) = iter.peek() {
      if next.starts_with(' ') || next.starts_with('\t') { iter.next(); } else { break; }
  }
  ```

  `yaml_scalar` delegates to `serde_yaml::to_string`, which renders a literal block scalar with **unindented empty lines**. Those fail the `starts_with(' ')` test, so consumption stops at the first blank line and the remaining tail of the old value survives below the newly written value. The next read (`extract_yaml_string_value`, via `serde_yaml`) sees that orphan as part of the existing value, so every subsequent write compounds.

  **Discriminating experiment (verified):**
  - Three appends of **single-line** notes: linear and correct — `AAA`, `BBB`, `CCC`, one key, 174 bytes.
  - Three appends of **realistic multi-line markdown** (heading, bullets, a table, blank lines): corrupted — final value is `AAA`, `BBB`, `CCC`, **`BBB`** with an orphaned tail at line 56, 992 bytes.
  The only variable is whether the value contains a blank line.

  **Why the real task reached 95.8 MB:** ~17 appends of multi-line notes, each leaving an orphan that the next read re-expanded. The observed ruler-like interleaving (`n1 n2 n1 n3 n1 n2 n1 n4 ...`) is the accumulated result of that, not a doubling operation in the append code.

  **Blast radius is wider than this task's title.** `set_yaml_field` has 17 call sites, writing `status`, `implementation_notes`, `implementation_plan`, `title`, `priority`, `assignee`, `id`, `type`, `time_started`, `time_spent`. Any **multi-line** value containing a blank line is only partially replaced — so `implementation_plan` is affected by the same defect, and so is any other free-text frontmatter field.

  **Correct fix location:** `set_yaml_field` must consume the whole existing value, treating empty and whitespace-only lines as continuation, and stopping only at a non-indented non-empty line (the next top-level key, or the closing `---`). It must stay line-based: this repo has a critical pattern against whole-block YAML round-trips (duplicate frontmatter, escaped newlines, ids becoming floats).

  **Secondary defect confirmed again:** `wm_task.get` reports `implementation_notes` as 0 characters even when the file clearly contains it, which is why this went unnoticed for so long.

  ## FIXED and independently verified (2026-10-15)

  `apps/wm-core/src/page/helpers/yaml_helper.rs` now extracts `is_block_continuation(line)` returning `indented || whitespace-only`, and the replacement loop in `set_yaml_field` consumes while that holds, stopping only at a non-indented, non-empty line. `yaml_scalar` is untouched — the defect was in the consumer, as diagnosed.

  Seven tests added in `apps/wm-core/src/page/mod.rs`: replacement without an orphan tail, second replacement not growing, multiline result parsing and preserving other keys, three appends producing three blocks, `implementation_plan` round-trip, single-line fields unchanged, plus the existing id-quoting and scientific-notation cases.

  **Verified by the reviewer, not from the lane report:** the 8 `set_yaml_field` unit tests pass, the three-append regression passes, the 33 CLI goldens pass on a freshly rebuilt binary, and `cli_test` is 17/17. The lane additionally demonstrated the live before/after: before the fix the notes region compounded orphaned fragments; after it, exactly `AAA`, `BBB`, `CCC` with no fourth block.

  **Latent same-class pattern left un-fixed and filed separately:** `remove_yaml_block`, `set_yaml_value_field` and `set_yaml_block` still stop at the first blank line. Their keys (`tags`, `acceptance_criteria`, `relates_to`, `answers`, JSON extra frontmatter) are renderer-produced and contain no blank lines, so no corruption has been observed — but the pattern is the same.

  ## ROOT CAUSE FOUND (2026-10-15) — it is not in append_notes at all

  Both `append_notes` implementations are correct: `wm_task.update` computes `existing + "\n" + append` and `page_update_builder_service.rs` does the same. The defect is in **`set_yaml_field`** (`apps/wm-core/src/page/helpers/yaml_helper.rs:111`), the line-based frontmatter replacer.

  **Mechanism.** `set_yaml_field` replaces a key by writing the new value and then consuming the old value with:

  ```rust
  while let Some(next) = iter.peek() {
      if next.starts_with(' ') || next.starts_with('\t') { iter.next(); } else { break; }
  }
  ```

  `yaml_scalar` delegates to `serde_yaml::to_string`, which renders a literal block scalar with **unindented empty lines**. Those fail the `starts_with(' ')` test, so consumption stops at the first blank line and the remaining tail of the old value survives below the newly written value. The next read (`extract_yaml_string_value`, via `serde_yaml`) sees that orphan as part of the existing value, so every subsequent write compounds.

  **Discriminating experiment (verified):**
  - Three appends of **single-line** notes: linear and correct — `AAA`, `BBB`, `CCC`, one key, 174 bytes.
  - Three appends of **realistic multi-line markdown** (heading, bullets, a table, blank lines): corrupted — final value is `AAA`, `BBB`, `CCC`, **`BBB`** with an orphaned tail at line 56, 992 bytes.
  The only variable is whether the value contains a blank line.

  **Why the real task reached 95.8 MB:** ~17 appends of multi-line notes, each leaving an orphan that the next read re-expanded. The observed ruler-like interleaving (`n1 n2 n1 n3 n1 n2 n1 n4 ...`) is the accumulated result of that, not a doubling operation in the append code.

  **Blast radius is wider than this task's title.** `set_yaml_field` has 17 call sites, writing `status`, `implementation_notes`, `implementation_plan`, `title`, `priority`, `assignee`, `id`, `type`, `time_started`, `time_spent`. Any **multi-line** value containing a blank line is only partially replaced — so `implementation_plan` is affected by the same defect, and so is any other free-text frontmatter field.

  **Correct fix location:** `set_yaml_field` must consume the whole existing value, treating empty and whitespace-only lines as continuation, and stopping only at a non-indented non-empty line (the next top-level key, or the closing `---`). It must stay line-based: this repo has a critical pattern against whole-block YAML round-trips (duplicate frontmatter, escaped newlines, ids becoming floats).

  **Secondary defect confirmed again:** `wm_task.get` reports `implementation_notes` as 0 characters even when the file clearly contains it, which is why this went unnoticed for so long.
---

**Bug description:** `wm_task.update` with `append_notes` does not append. It reproduces the entire accumulated notes block and re-inserts the new note, so the notes grow as a ruler/prefix sequence and the file size grows as **2^n** in the number of appends. A task page I updated ~17 times reached **95.8 MB**, and its version record reached **167.0 MB**.

**Tool name:** `wm_task` (action `update`, parameter `append_notes`). Same growth is visible in the generated version record, so the version-store path inherits it.

**Reproduction:**
1. Create a task, then call `wm_task({"action":"update","id":"<task>","append_notes":"## note k"})` repeatedly, ~17 times.
2. Observe `implementation_notes` in the file's frontmatter grows super-linearly, not linearly.
3. After ~17 appends the file is ~96 MB and the version JSON ~167 MB.

**Measured evidence (this repo, 2026-10-15):**
- `wiki:tasks:controlled-rewrite-of-appswm-clisrcmainrs--one-architecture-decision-per-review`: file 95.8 MB; version record 167.0 MB.
- Occurrences of one note heading written on the ~4th append: **4,096 = 2^12**. One written on the ~5th: **8,192 = 2^13**. Total `## ` headings in the notes block: **65,791**.
- The observed order is the ruler sequence `n1 n2 n1 n3 n1 n2 n1 n4 n1 n2 n1 n3 n1 n2 n1 …`, i.e. append *k* of *n* ends up duplicated **2^(n-k)** times, and the total is 2^n blocks. With ~3 KB notes and 17 appends that is ~96 MB.

**Full input parameters:** `{"action":"update","id":"wiki:tasks:controlled-rewrite-of-appswm-clisrcmainrs--one-architecture-decision-per-review","append_notes":"<markdown>"}`, repeated.

**Full output:** `{"id":"…","status":"updated"}` every time. **This is the worst part:** each call reported success while writing exponentially duplicated content, so the defect is invisible to the caller and to any check that only reads the returned status.

**Impact:**
- A wiki page reached 96 MB, which made every subsequent read/index/validate touch it.
- It **blocked `git push`**: GitHub rejected the branch with `GH001: Large files detected` because the 96 MB page and 167 MB version record were committed.
- Unbounded growth means a long-running task record cannot be updated safely at all.

**Workaround used to repair:** split the notes block on `^  ## ` headings, keep the first occurrence of each heading, and reassemble in first-seen order. Result: 65,791 sections collapsed to **19 unique notes**, 95.8 MB to **34.2 KB**, with `wm_validate.check` passing afterwards. The corrupt 167 MB version record was deleted rather than repaired.

**Secondary defect found at the same time:** `wm_task.get` reports `implementation_notes` as **0 characters** for a page whose frontmatter clearly contains it. The field appears to be write-only through the API, which is why the bloat was not noticed for so long.

**Fix direction:**
1. Make `append_notes` actually append (read-modify-write of just the new block), not concat-and-duplicate.
2. Add a sanity bound: refuse a notes write that more than doubles the field, and return an error rather than `status: updated`.
3. Version records must not store a full notes snapshot per version if that snapshot is itself growing super-linearly.
4. Surface `implementation_notes` from `wm_task.get` so the content is observable.
5. Add a regression test asserting that N sequential appends produce exactly N note blocks and linear size growth.