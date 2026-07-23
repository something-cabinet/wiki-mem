---
title: Edge type pruning — inverse-edge policy + graceful degredation
type: memory
tags: [edge-types, pruning, pattern]
created_at: "2026-07-20"
relates_to:
  - {type: references, target: wiki:concepts:edge-types}
---

## Inverse-edge policy

Pick one canonical direction per relationship; don't define inverses. `petgraph` traverses incoming edges natively. Saves 2+ enum variants per relationship pair. Documented in `@wiki/concepts/edge-types`.

## Graceful degradation for pruned enum variants

When removing enum variants from a `Custom`-backed type, the lenient parser should map old names to `Custom` rather than rejecting them. This avoids data migration on existing frontmatter. `parse_edge_type_flexible` in `relation_helper.rs` demonstrates the pattern.

## Audit hygiene

Before pruning, verify usage claims against real frontmatter — doc examples in YAML code blocks look like real usage but aren't. The original audit was wrong in both directions (claimed `extends` had usage, missed `part_of`).
