---
title: 'Pattern: Line-based YAML frontmatter editing'
type: pattern
id: wiki:patterns:line-based-frontmatter-editing
status: reviewed
tags:
- pattern
- wiki
- yaml
- frontmatter
relates_to:
  - {type: implements, target: wiki:specs:record-format-wave}
---

schema_version: 1
state: |-
  ## Problem

  Editing YAML frontmatter by parsing the whole block with serde_yaml and re-serializing it silently corrupts data: unquoted numeric-looking values (like 6-char hex IDs `652e07`) get re-interpreted as floats, unmodeled fields get dropped, and empty maps emit `{}` blocks. This caused a real corruption bug in wiki-mem (see the failure concept page).

  A second failure mode surfaced 2026-08-14 (issue #126 wave): **raw `format!`-built frontmatter that never quotes user-supplied scalars**. Titles starting with `[` (flow-sequence syntax), containing `: ` (mapping syntax), or containing backslashes produce **unparseable YAML**, and the task store silently drops the file ("task not found") until the frontmatter is fixed by hand.

  A third failure mode surfaced 2026-10-21: **block consumption stopped at the first blank line or non-indented list item**. A literal block scalar renders blank lines unindented, and valid YAML permits a bare `-`/`*` list at column 0. A replacer that only skips indented lines leaves the old value's tail behind; the next read re-expands it, so repeated appends grow super-linearly. `set_yaml_field` had this defect for multi-line notes; `remove_yaml_block`, `set_yaml_value_field` and `set_yaml_block` had the same gap for tags/ACs.

  ## Solution

  Edit frontmatter **line-based**, never whole-block:

  - `set_yaml_field(yaml, key, value)` — replace only the target top-level line and its continuation, preserving every other line byte-for-byte.
  - `remove_yaml_block(yaml, key)` — drop a key plus its continuation without touching the rest.
  - `set_yaml_value_field(yaml, key, &json)` — the single writer for lists, maps, and scalars; use it for tags, acceptance criteria, relates_to, and any `extra_frontmatter`.
  - `ac_set_checked` — set `checked:` on the Nth acceptance-criteria item by scanning lines, not by parsing the block.
  - **Continuation means more than indentation.** Treat a line as part of the current value when it is indented, blank, or a bare `-`/`*` list item at column 0. Stop only at a non-indented, non-empty line that starts the next top-level key (or the closing `---`). Share one predicate across all block consumers.
  - **Every write path uses the writer, including updates.** Do not build `tags: [..]` or `- {text: ...}` with `format!`; route through `set_yaml_value_field` so quoting and block shape are guaranteed.
  - Always **double-quote `id`** (`id: "652e07"`) everywhere it is written.
  - **Quote (or YAML-escape) any user-supplied scalar YAML could misread** — leading `[`, `: ` inside the value, backslashes. Prefer a YAML-aware serializer for the value.
  - On unparseable YAML, preserve the original rather than wiping it; never emit a `{}` block for an empty map.

  Reference implementation: `packages/wm-engine/src/helpers/yaml_helper.rs` (line-based writer + shared `is_block_continuation`), `packages/wm-engine/src/parser/mod.rs` (frontmatter health validator rules).

  ## When to Use

  Any code that reads, modifies, and rewrites YAML frontmatter — task pages, doc pages, memory entries, config files, migration scripts. Especially writers that interpolate user/agent-supplied titles, tags, or acceptance criteria into frontmatter, and any repeated append operation over multi-line values.

  ## When Not to Use

  - Full-file YAML files where you genuinely re-serialize the whole document (but still quote ids).
  - One-off manual edits (a migration script should mirror the same rules).

  ## Related

  - @wiki/patterns/durable-stale-marker-for-failed-refresh — sibling reliability pattern for derived-state writes
  - @wiki/specs/record-format-wave
  - @wiki/tasks/same-latent-block-consumption-pattern-in-removeyamlblock-setyamlvaluefield-and-setyamlblock
  - @wiki/tasks/wmpageupdate-corrupts-frontmatter-on-non-indented-yaml-list-fields
  - @wiki/tasks/wiki-tool-reliability-wmtaskupdate-appendnotes-duplicates-the-notes-block-exponentially
  - @wiki/tasks/frontmatter-serializer-quoting-for-task-titles-and-acs
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
answers:
  problem_kind: tooling
  preconditions_required: true
  complexity: moderate
  language_specific: false