---
title: Frontmatter block replacement must treat blank lines as continuation
type: memory
tags: [frontmatter, yaml, data-corruption, parser-format]
status: active
---

schema_version: 1
state: |-
  `set_yaml_field` replaced a key by writing the new value and then consuming the old one only while lines started with a space or tab. `yaml_scalar` renders a literal block scalar with UNINDENTED empty lines, so consumption stopped at the first blank line and the tail of the old value survived below the new one. The next read included that orphan, so every later write compounded: a task page reached 95.8 MB after about seventeen appends, its notes region becoming the ruler sequence n1 n2 n1 n3 n1 n2 n1 n4 ...

  Fixed by `is_block_continuation` (indented OR whitespace-only) in `apps/wm-core/src/page/helpers/yaml_helper.rs`, which stops only at a non-indented, non-empty line.

  Diagnostic that isolates it in one step: three appends of single-line content stay linear (174 bytes); three appends of multi-line markdown containing a blank line corrupt (992 bytes, a duplicated tail). The only variable is the blank line.

  The defect is in the REPLACER, not in `append_notes` — both append implementations already computed `existing + newline + append` correctly, which is where the first investigation looked.

  Latent in three siblings, filed separately: `remove_yaml_block`, `set_yaml_value_field`, `set_yaml_block`.

  Refs: @wiki/patterns/line-based-frontmatter-editing, @wiki/tasks/wiki-tool-reliability-wmtaskupdate-appendnotes-duplicates-the-notes-block-exponentially
questions:
  - id: layer
    type: choice
    instructions: Which memory layer does this entry belong to?
    options:
    - project
    - global
    - session
  - id: store_or_skip
    type: noul
    instructions: This entry is worth storing as durable memory.
  - id: dedup_action
    type: choice
    instructions: How should this entry relate to existing memory?
    options:
    - new
    - merge
    - supersede
    - skip
  - id: confidence
    type: score
    instructions: How confident is the recorded knowledge?
    levels:
    - low
    - medium
    - high
answers: {}
