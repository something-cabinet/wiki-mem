---
title: 'Failure: Frontmatter corruption — unquoted id parsed as scientific notation'
type: concept
id: wiki:concepts:frontmatter-corruption-sci-notation-id
status: draft
tags:
- failure
- wiki
- yaml
- frontmatter
relates_to:
  - {type: references, target: wiki:tasks:wm-task-update-frontmatter-corruption}
---

## What went wrong

Task and page IDs stored unquoted in YAML frontmatter (e.g. `id: 652e07`) were silently rewritten to `id: 6520000000.0` — the 6-char ID `652e07` was parsed as scientific-notation float (6.52×10⁹) and re-serialized. One real task file (`652e07.md`) was corrupted this way; the same bug family stripped `id:/title:/type:` from other task files during `wm_task.update`, and emitted bare `{}` frontmatter blocks.

## Root cause

Any write path that round-trips the **entire frontmatter block** through `serde_yaml::from_str → to_string` re-interprets unquoted scalar values:
- `652e07` → parsed as float → re-emitted as `6520000000.0`
- Whole-block YAML round-trips drop fields not modeled by the local struct (field stripping)
- Empty-map serialization emits `{}` blocks

The old `set_yaml_field` / `ac_set_checked` / timer-recovery / doc-write paths all did whole-block round-trips.

## Prevention

- **Never round-trip a whole frontmatter block through serde_yaml for a field edit** — use line-based helpers (`set_yaml_field`, `remove_yaml_block`, `ac_set_checked` in `apps/wm-core/src/page/helpers/yaml_helper.rs`) that preserve every other line byte-for-byte.
- **Always double-quote `id`** in frontmatter (`id: "652e07"`) — a quoted string survives any round-trip.
- Validator/lint rules catch the three corruption shapes: sci-notation-shaped unquoted ids (`^[0-9]+e[0-9]+$`), duplicate frontmatter blocks, and task filename↔id mismatch.

## Time lost

Multiple hours across task `wm-task-update-frontmatter-corruption`: root-cause tracing, 33-file repair, write-path rewrite, validator rules, regression tests.

## Related

- @task-wm-task-update-frontmatter-corruption
- Pattern: line-based frontmatter editing