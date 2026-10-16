---
title: 'Pattern: Verify writes by read-back'
type: pattern
id: wiki:patterns:verify-writes-by-read-back
status: draft
tags:
- pattern
- tool-reliability
- persistence
- testing
relates_to:
  - {type: references, target: wiki:tasks:wiki-tool-reliability-wmpageupdate-returns-ioerror-on-success}
---

schema_version: 1
state: |-
  ## Problem

  A write path can return `Ok` while persisting nothing. In wiki-mem, `wm_task check_ac` returned `{"checked":[0]}` for six calls across two tasks and wrote zero `checked: true` to disk; `wm_task update` returned `{"status":"updated"}` while the page kept `status: todo`, and the version record logged changes the page never received. The handler trusted the write helper's return value, and the write helper had a guard that returned its input unchanged.

  ## Solution

  Treat "returned `Ok`" as unproven. After the write, re-read the file from disk, parse it, and assert the specific field(s) changed. Return the success payload only after the read-back confirms the change; otherwise return an error.

  ```rust
  page::update_page(engine, &id, &params)?;
  verify_ac_checked(&meta.path, index, checked)?;
  store.save_task_version(&id, changes)?;
  Ok(json!({ "id": id, "checked": indices }))
  ```

  ```rust
  fn verify_ac_checked(path: &Path, index: usize, expected: bool) -> Result<(), ToolError> {
      let content = std::fs::read_to_string(path)
          .map_err(|e| ToolError::io_error("read", path.display().to_string(), e))?;
      let (frontmatter, _) = crate::parser::extract_frontmatter(&content);
      let actual = frontmatter
          .and_then(|fm| fm.acceptance_criteria.into_iter().nth(index))
          .map(|criterion| criterion.checked);
      match actual {
          Some(value) if value == expected => Ok(()),
          _ => Err(ToolError::internal(format!(
              "Acceptance criterion {} in {} was not persisted with checked = {}",
              index, path.display(), expected
          ))),
      }
  }
  ```

  Supporting rules that made the bug possible, and which the pattern closes:

  - **Bounds-check input before writing.** Out-of-range indices must error, not succeed silently.
  - **Order side effects after the source-of-truth write.** Version/index records are written only once the page write returns `Ok`.
  - **No-op guards must not report success.** A helper that returns its input unchanged on some branch is a silent failure when a change was requested.

  ## When to Use

  - Every tool/handler whose contract is "this call persisted a change".
  - Any write helper that can hit a no-op, guard, or early-return branch.
  - Any path that also writes a derived store (version history, index, cache) that a later reader will trust.

  ## When Not to Use

  - Pure in-memory calculations with no durable backing store.
  - Read-only operations.

  ## Testing note

  Assert on the ARTIFACT, not on the response and not on pre-existing content. The original `test_ac_set_checked` was a false positive: its input already contained `checked: true` elsewhere, so `contains("checked: true")` passed even though the function inserted nothing. Regression tests must write a real page, call the tool, then re-read and parse the markdown from disk.

  ## Related

  - @wiki/rules/durable-write-or-error
  - @wiki/concepts/failure-silent-no-op-write-helper
  - @wiki/specs/wmtask-checkac-persistence-fix

  ## 2026-10-15 — the inverse case: an error payload for a write that landed

  This pattern is recorded for the case of a **success payload for a write that never happened**. The mirror case was observed twice this session: `wm_page.update` returned `{"code":"IO_ERROR","error":"No such file or directory (os error 2)"}` while the write had **fully succeeded** — file grown, appended content present, envelope valid and correctly ordered, and `wm_page.get` parsing it with no `record_format_hint` warning.

  So verify by read-back in **both** directions, and treat neither response as authoritative. An agent that trusts the error will either retry (duplicating content) or report failure for completed work — the second failure direction, and arguably worse than a false success.

  Source: @wiki/tasks:wiki-tool-reliability-wmpageupdate-returns-ioerror-on-success
questions:
  - id: problem_kind
    type: choice
    instructions: What kind of problem does this pattern solve?
    options:
    - architecture
    - api-design
    - data-model
    - error-handling
    - performance
    - testing
    - ui
    - tooling
    - workflow
  - id: preconditions_required
    type: noul
    instructions: This pattern requires specific preconditions to be met.
  - id: complexity
    type: score
    instructions: How complex is applying this pattern?
    levels:
    - trivial
    - simple
    - moderate
    - complex
  - id: language_specific
    type: noul
    instructions: This pattern is specific to a programming language.
answers: {}