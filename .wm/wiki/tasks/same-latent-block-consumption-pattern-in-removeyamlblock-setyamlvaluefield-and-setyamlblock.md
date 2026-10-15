---
title: Same latent block-consumption pattern in remove_yaml_block, set_yaml_value_field and set_yaml_block
type: task
id: "wiki:tasks:same-latent-block-consumption-pattern-in-removeyamlblock-setyamlvaluefield-and-setyamlblock"
status: todo
priority: medium
tags: [tool-reliability, parser-format, latent, frontmatter]
acceptance_criteria:
  - text: "remove_yaml_block, set_yaml_value_field and set_yaml_block all consume blank lines as part of the existing value, via the shared is_block_continuation predicate"
  - text: "Each helper has a test that plants a blank line inside the value and asserts no orphaned tail survives a replacement"
  - text: "No helper switches to a whole-block YAML round-trip; the line-based approach is preserved"
  - text: "The eight existing set_yaml_field tests and the CLI goldens stay green; no comments, no #[allow], functions at most 20 lines"
---

Found while fixing the verified corruption in `set_yaml_field` (`@wiki/tasks/wiki-tool-reliability-wmtaskupdate-appendnotes-duplicates-the-notes-block-exponentially`).

**Finding:** three sibling helpers in `apps/wm-core/src/page/helpers/yaml_helper.rs` stop consuming a value at the first blank line, exactly the defect that corrupted `implementation_notes`:

- `remove_yaml_block`
- `set_yaml_value_field`
- `set_yaml_block`

**Why it has not bitten yet:** the keys they handle — `tags`, `acceptance_criteria`, `relates_to`, `answers`, and the arbitrary JSON extra frontmatter emitted by `extra_frontmatter` — are produced by renderers that do not emit blank lines inside a block. So no observed corruption.

**Why it should still be closed:** the guard is the same one-line predicate that was just added (`is_block_continuation`). A future renderer that emits a blank line inside `acceptance_criteria` or `relates_to` would silently reproduce the same corruption — a partially replaced block with an orphaned tail — and it would surface as duplicated frontmatter rather than a loud failure. This is a latent variant of a bug that already cost a 95.8 MB page.

**Fix direction:** route all four helpers through the shared `is_block_continuation` predicate, with a test per helper that plants a blank line inside the value and asserts no orphan survives and a second write does not grow the block. Keep the line-based approach; do not switch to whole-block YAML round-trips.