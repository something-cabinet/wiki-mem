---
title: 'Pattern: Line-based YAML frontmatter editing'
type: pattern
id: wiki:patterns:line-based-frontmatter-editing
status: draft
tags:
- pattern
- wiki
- yaml
- frontmatter
relates_to:
  - {type: references, target: wiki:tasks:wmdoc-typetags-frontmatter-persistence}
---

---
title: 'Pattern: Line-based YAML frontmatter editing'
type: pattern
id: wiki:patterns:line-based-frontmatter-editing
status: draft
tags:
- pattern
- wiki
- yaml
- frontmatter
relates_to:
  - {type: part_of, target: wiki:core:critical-patterns}
---

## Problem

Editing YAML frontmatter by parsing the whole block with serde_yaml and re-serializing it silently corrupts data: unquoted numeric-looking values (like 6-char hex IDs `652e07`) get re-interpreted as floats, unmodeled fields get dropped, and empty maps emit `{}` blocks. This caused a real corruption bug in wiki-mem (see the failure concept page).

A second failure mode surfaced 2026-08-14 (issue #126 wave): **raw `format!`-built frontmatter that never quotes user-supplied scalars**. The task writer (`apps/wm-core/src/mcp/tools/task/mod.rs`) writes `title: {title}` and AC lists via string formatting; titles starting with `[` (flow-sequence syntax), containing `: ` (mapping syntax), or containing backslashes produce **unparseable YAML**, and the task store silently drops the file ("task not found") until the frontmatter is fixed by hand.

## Solution

Edit frontmatter **line-based**, never whole-block:

- `set_yaml_field(yaml, key, value)` — replace only the target top-level line (and its indented continuation), preserving every other line byte-for-byte.
- `remove_yaml_block(yaml, key)` — drop a key + its indented children without touching the rest.
- `ac_set_checked` — set `checked:` on the Nth acceptance-criteria item by scanning lines, not by parsing the block.
- Always **double-quote `id`** (`id: "652e07"`) everywhere it's written — quoted strings are immune to scalar re-interpretation.
- **Quote (or YAML-escape) any user-supplied scalar YAML could misread** — leading `[`, `: ` inside the value, backslashes. Prefer writing values through a YAML-aware serializer (e.g. `serde_yaml::to_string` for the value, or quote it) rather than raw `format!("title: {}")`; the `wm_page` writer (page/action.rs + yaml_helper) already does this correctly — the task store should reuse the same helper family.
- On unparseable YAML, preserve the original rather than wiping it; never emit a `{}` block for an empty map.

Reference implementation: `apps/wm-core/src/page/helpers/yaml_helper.rs` (line-based), `apps/wm-core/src/parser/mod.rs` (`inspect_frontmatter_health` validator rules).

## When to Use

Any code that reads, modifies, and rewrites YAML frontmatter — task pages, doc pages, memory entries, config files, migration scripts. Especially writers that interpolate user/agent-supplied titles, tags, or acceptance criteria into frontmatter.

## When Not to Use

- Full-file YAML files where you genuinely re-serialize the whole document (but still quote ids).
- One-off manual edits (a migration script like `scripts/migrate-wiki-frontmatter-0.4.3.py` should mirror the same rules).

## Related

- @task-wm-task-update-frontmatter-corruption
- Failure: frontmatter corruption — sci-notation id
- @task-frontmatter-serializer-quoting-for-task-titles-and-acs (task-store quoting gap found 2026-08-14)
- `scripts/migrate-wiki-frontmatter-0.4.3.py` — codified migration implementing these rules