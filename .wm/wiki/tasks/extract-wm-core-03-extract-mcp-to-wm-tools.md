---
title: "EXTRACT-03 Extract mcp/** to packages/wm-tools"
id: "wiki:tasks:extract-wm-core-03-extract-mcp-to-wm-tools"
type: task
status: todo
priority: high
tags: [from-spec, spec:wm-core-layer-extraction, refactor, wm-tools, mcp]
acceptance_criteria:
  - text: "AC-2: apps/wm-core/src/mcp/ no longer exists; wm-core source is 12,000 lines or fewer, measured"
  - text: "packages/wm-tools exists and declares no rmcp and no clap; error/ stays in wm-core"
  - text: "AC-7: tools/code.rs delegates file walking and language inference to wm-code-intel; wm-tools adds no new walkdir/ignore traversal"
  - text: "cargo build, test and clippy --workspace are green"
implementation_notes: |-
  EXTRACT-03 review + P2 fix, 2026-10-16:
  - AC-2 verified: apps/wm-core/src/mcp gone; wm-core 11,771 lines (<=12,000, src-only convention). AC-7 verified: wm-tools has 0 rmcp/clap; tools/code.rs delegates walk/language to wm-code-intel; the remaining walkdir in tools/lint.rs:195 and tools/lsp.rs:247 is relocated pre-existing code, byte-identical to HEAD.
  - Owner rulings: (1) approved that wm_code.search/symbols/deps prune .wm and wm_code.file denies .wm reads - recorded on @wiki/tasks/wmcode-read-policy-should-follow-gitignore-not-a-hardcoded-hidden-file-ban; (2) extend wm-code-intel's extension->language mapping to restore rich labels (javascript/jsx/css/scss/json/yaml/md) - dispatched.
  - P2 fixed: MCP integration tests relocated to packages/wm-tools/tests (mcp_test, e2e_mcp, semantic_test, golden_eval, security_test, graph_code_edges, error_data_transport_test + helpers/inproc.rs, helpers/mcp.rs); wm-core's wm-tools dev-dependency removed; unused ignore/schemars dropped; crate description updated.
  - FINDING (P3, open): tests/helpers/setup.rs is now DUPLICATED - an identical copy exists at apps/wm-core/tests/helpers/setup.rs and packages/wm-tools/tests/helpers/setup.rs, since 12 wm-core tests and the moved wm-tools tests both need it. Silent drift risk; resolve via a shared test-support strategy or accept deliberately.
  - FINDING (pre-existing): cargo test -p wm-tools with default features (onnx on) runs semantic_test's model-dependent tests, which fail without a downloaded model. CI runs the onnx-off feature set and never hits them; reproduced identically in the original wm-core location.
---

schema_version: 1
state: |-
  D-6 step 2: extract `mcp/**` (8,789 lines) to `packages/wm-tools`. `ToolError`/`error/` stays behind with its 36 consumers (D-8). wm-tools declares no `rmcp` and no `clap` - it is transport-free (D-5).

  The import-rewrite blast radius is roughly 30 cross-boundary paths; split file-moves first, import-rewrite second, never half the tools moved. `tools/code.rs` must delegate traversal/language inference to wm-code-intel rather than arrive with its own walk. Part of @wiki/specs/wm-core-layer-extraction.
questions:
  - id: work_kind
    type: choice
    instructions: What kind of work is this task?
    options:
    - feature
    - bugfix
    - refactor
    - docs
    - test
    - chore
    - migration
  - id: priority
    type: choice
    instructions: What priority is this task?
    options:
    - low
    - medium
    - high
    - urgent
  - id: needs_spec
    type: noul
    instructions: This task depends on a spec.
  - id: has_ac
    type: noul
    instructions: This task has at least one acceptance criterion.
answers: {}