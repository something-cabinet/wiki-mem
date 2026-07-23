---
title: Investigate and resolve wiki graph cycle
type: task
status: todo
---

---
title: Investigate and resolve wiki graph cycle
type: task
status: done
spec: specs/graph-bugs-review-fixes
task_data:
  acceptance_criteria:
    - text: "Graph cycle either resolved or documented as intentional"
      checked: true
    - text: "No \"Cycle detected in wiki graph\" warning on startup (or explicit note if expected)"
      checked: true
relates_to:
  - {type: implements, target: wiki:specs:graph-bugs-review-fixes}
---

**Severity:** Low

**Observed:** A cycle was detected in the wiki graph during rebuild. BFS uses visited tracking to prevent infinite loops.

**Impact:** Graph traversal (e.g. neighbors, path finding) may produce unexpected results around the cycle. The cycle itself isn't harmful but indicates a circular reference in `relates_to` frontmatter.

## Investigation Results

Two cycles were found, both **intentional mutual references**:

### Cycle 1: `patterns:mcp-response-format` ↔ `patterns:rust-binary-integration-test`
- `patterns:mcp-response-format` → `patterns:rust-binary-integration-test` (`references`)
- `patterns:rust-binary-integration-test` → `patterns:mcp-response-format` (`references`)
- Both patterns are about MCP/integration testing and naturally reference each other as related content. Breaking either direction would lose a valid cross-reference.

### Cycle 2: `specs:engine-explicit-project-root` ↔ `tasks:engine-explicit-project-root`
- `specs:engine-explicit-project-root` → `tasks:engine-explicit-project-root` (`references`)
- `tasks:engine-explicit-project-root` → `specs:engine-explicit-project-root` (`implements`)
- Standard spec↔task pattern: the spec references the task it defines, the task implements the spec. This is intentional and common across the wiki.

### Conclusion
Both cycles are benign mutual references. The `petgraph::algo::is_cyclic_directed` check flags all directed cycles, including intentional bidirectional edges. Per spec.md FR-5, cycles are diagnostic only — the graph is never mutated. BFS traversal uses visited tracking (`HashSet<NodeIndex>`) to prevent infinite loops, so cycles have no operational impact.

The `info!()` log message at `apps/wm-core/src/graph/mod.rs:142` already notes: *"Cycle detected in wiki graph (expected: mutual relates_to links)."*

**No code changes needed.** The cycles are documented here for awareness.

**Acceptance Criteria:**
- [x] Graph cycle either resolved or documented as intentional
- [x] No "Cycle detected in wiki graph" warning on startup (or explicit note if expected) — the log message is `info!()` level, not a warning, and is expected behavior