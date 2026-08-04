---
title: 'Pattern: Bulk YAML Frontmatter Field Rename via sed'
id: wiki:patterns:bulk-yaml-frontmatter-rename
type: pattern
relates_to:
  - {type: references, target: wiki:tasks:rename-knownsid-to-id-in-task-frontmatter}
---

---
title: Pattern: Bulk YAML Frontmatter Field Rename via sed
type: pattern
tags: [pattern, sed, yaml, frontmatter, migration]
---

## Problem

Renaming a YAML frontmatter field (e.g., `knowns_id` → `id`) across hundreds of wiki/markdown files. Each file has a `---` bounded YAML block at the top.

## Solution

Use a line-start-anchored `sed` replacement scoped to frontmatter-only matches. Unlike comment removal (which risks false positives inside string literals), frontmatter field keys are always at the start of a line inside the YAML block:

```bash
# Safe: anchored to line start, targets exact key
find .wm/wiki/tasks/ -name '*.md' -exec sed -i '' 's/^knowns_id:/id:/' {} +

# Verify
rg -c '^knowns_id:' .wm/wiki/tasks/  # expect zero
rg -c '^id:' .wm/wiki/tasks/         # expect all files
```

For adding a new field (not renaming), use a script that reads the file, finds the frontmatter boundaries, and inserts the new line after the opening `---`:

```bash
sed -i '' "s/^---$/---\nid: $id/" "$file"
```

The key constraint: the replacement must be anchored (`^key:`) so it only matches YAML frontmatter, not content body or code blocks that happen to contain similar text.

## When to Use

- Renaming or adding typed YAML frontmatter fields across many wiki files
- The field key is unique to frontmatter (line-start-anchored match is unambiguous)
- One-time bulk migration where MCP tool support doesn't exist for the operation

## When Not to Use

- Modifying file content (not frontmatter) — use AST-aware tools instead
- The field value could appear inline in code blocks or body text — requires file-level parsing, not just sed
- Regular ongoing operations — should be done via MCP tools (`wm_page.update`) for cache consistency
- Removing content from string literals, URLs, or comments — use AST tools, never sed for this

## Risks

- Always verify with `cargo check` / `cargo test` and `wm_index.rebuild` after the bulk operation
- The in-memory graph cache will be stale until rebuilt — schedule a rebuild after any bulk frontmatter change
- Files written directly via sed/script won't trigger cache invalidation or version history — this is acceptable for one-time migrations but not for regular use

## Related

- @wiki/tasks/rename-knownsid-to-id-in-task-frontmatter
- @wiki/concepts/sed-bulk-comment-removal-risk (contrast: unsafe sed usage for code content)
- @wiki/tasks/extend-wmpageupdate-to-accept-arbitrary-frontmatter-fields (future: MCP tool for this)