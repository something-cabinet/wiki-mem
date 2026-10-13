---
title: 'Pattern: Atomic file writes with unique temp names and per-path locks'
type: pattern
id: wiki:patterns:atomic-file-writes-unique-temp-and-path-lock
status: draft
tags: [pattern, tool-reliability, concurrency, persistence]
relates_to:
  - {type: references, target: wiki:tasks:wmpagelink-concurrency-race-and-relates-to-clobber}
  - {type: references, target: wiki:rules:durable-write-or-error}
---

schema_version: 1
state: |-
  ## Problem

  Two concurrent writes to the same file corrupt or fail. In wiki-mem, `FsPageRepo::write` used a temp name derived only from the target name and the process id (`".{file}.tmp-{pid}"`). Two concurrent `wm_page.link` calls in one process therefore wrote to the SAME temp path; the first `std::fs::rename` consumed it and the second failed with `ENOENT` — surfaced as `IO_ERROR "No such file or directory (os error 2)"`. Even when the write itself succeeds, a naive read-modify-write loses updates: two writers read the same base and the last one wins, silently dropping the other's change.

  ## Solution

  1. **Unique temp per call.** Build the temp name from the target name plus a per-process monotonic counter, not just the pid.
     ```rust
     static WRITE_SEQUENCE: AtomicU64 = AtomicU64::new(0);
     let tmp = parent.join(format!(
         ".{}.tmp-{}-{}",
         file_name, std::process::id(), WRITE_SEQUENCE.fetch_add(1, Ordering::Relaxed)
     ));
     std::fs::write(&tmp, content)?;
     std::fs::rename(&tmp, path)
     ```
     `rename` stays atomic; two writers no longer share a temp path.
  2. **Serialize the read-modify-write per resource.** A process-wide map of mutexes keyed by the canonical path, held across the whole read-modify-write, so a concurrent pair cannot both read the same base and clobber each other.
     ```rust
     static PAGE_WRITE_LOCKS: LazyLock<Mutex<HashMap<PathBuf, Arc<Mutex<()>>>>> = ...;
     ```
  3. **Merge append-only fields.** A field like `relates_to` is a set; the write path must merge with what is on disk (dedupe), not replace it with the new element.

  Do heavy post-write work (graph update, index refresh, notification) OUTSIDE the lock.

  ## When to Use

  - Any file that more than one tool call in the same process can write.
  - Append-only or set-valued frontmatter fields (`relates_to`, `tags`, `sources`).
  - Any write path where "read the current value, change part of it, write it back" happens.

  ## When Not to Use

  - Immutable-once-written files with a single writer.
  - Pure in-memory state with its own synchronization.

  ## Testing note

  A concurrency test must fire N parallel writers at one target and assert N durable results (and no error), not just that each call returned success. Run it repeatedly — a single green run does not prove a race is fixed. Also test sequential repeat calls: merge bugs show up without any concurrency.

  ## Related

  - @wiki/tasks/wmpagelink-concurrency-race-and-relates-to-clobber
  - @wiki/decisions/link-merge-vs-update-replace
  - @wiki/patterns/verify-writes-by-read-back
  - @wiki/rules/durable-write-or-error
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
