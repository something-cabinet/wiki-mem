---
{}
---

id: wiki:patterns:numbered-steps-to-functions

## Problem

Numbered step comments like `// 1. Get custom types from config`, `// 2. Full graph rebuild`, `// 3. Regenerate index.md` explain the structure of a function but are invisible to the compiler, rot when steps are reordered, and don't compose.

## Solution

Extract each numbered step into its own named private function. The step sequence becomes the sequence of function calls:

```rust
// Before:
fn rebuild_graph(...) {
    // 1. Get custom types from config
    let ct = config.read().unwrap().custom_edge_types.clone();
    // 2. Full graph rebuild
    let count = rebuild_graph_snapshot(...);
    // 3. Regenerate index.md
    regenerate_index(wiki_dir);
    // 4. Parse sections for this single file
    let sections = build_sections_from_wiki(wiki_dir);
    // 5. Update section corpus atomically
    section_corpus.store(Arc::new(sections));
    // 6. Rebuild BM25 index from updated corpus
    let bm25 = Bm25Index::build(docs);
    bm25_index.store(Arc::new(bm25));
}

// After:
fn rebuild_graph(...) {
    let ct = get_custom_types_from_config();
    let count = rebuild_graph_snapshot(...);
    regenerate_index_md(wiki_dir);
    let sections = parse_sections(wiki_dir);
    update_section_corpus(sections);
    rebuild_bm25_index(docs);
}
```

### Benefits
- Each step is independently testable
- Steps can be reused across callers
- Reordering is a simple call sequence change
- The function name documents what the step does (no comment needed)
- The compiler validates signatures — comments can't drift

## When to Use
- Any function with 3+ numbered/bulleted step comments
- Sequential processing pipelines (rebuild, migrate, transform)
- Long functions where each phase is a clear conceptual unit

## When Not to Use
- Single-line operations that are already a named function call
- Steps that share mutable state so tightly that extraction requires excessive parameter passing

## Related
- @wiki/tasks/c19d50