---
title: wm_doc type/tags frontmatter persistence
type: task
id: wiki:tasks:wmdoc-typetags-frontmatter-persistence
status: done
priority: high
tags:
- from-spec
- spec:wm-doc-type-frontmatter
- wm-doc-fix-01
spec: wiki:specs:wm-doc-type-frontmatter
acceptance_criteria:
- text: 'AC-1: wm_doc.create({path, title, type: "spec", content}) writes .wm/wiki/<path>.md whose frontmatter contains type: spec'
  checked: true
- text: 'AC-2: wm_doc.update({path, type: "howto"}) rewrites frontmatter type and preserves title + body'
  checked: true
- text: 'AC-3: wm_doc.update({path, tags}) persists tags; existing type preserved when not provided'
assignee: '@me'
time_started: 2026-08-14T06:53:56.865504+00:00
implementation_plan: |-
  ## Implementation Plan — wm_doc type/tags frontmatter persistence

  ### Approach
  Fix `wm_doc` (apps/wm-core/src/mcp/tools/doc.rs) to full parity with `wm_page`: wire the declared-but-dead `r#type` through create, add `type` + `tags` to update, and remove the `#[allow(dead_code)]` annotation (D3 dead_code ban). Root cause verified: `r#type: _` destructure (doc.rs:157), `build_markdown(&title, &content, &tags)` (doc.rs:187), Update schema lacks type/tags (doc.rs:40-44).

  ### Steps
  1. **Create path (doc.rs)** — remove `#[allow(dead_code)]` (doc.rs:35); destructure `r#type` instead of `_` (doc.rs:157); extend `build_markdown` to serialize `type` into frontmatter (doc.rs:187). When `type` is absent, derive it from the path directory (match `wm_page` default — FR-3).
  2. **Update path (doc.rs)** — add `type: Option<String>` + `tags: Option<Vec<String>>` to `WmDocAction::Update` (doc.rs:40-44); merge into the frontmatter map in the handler (doc.rs:201-252) exactly like `title` is merged (doc.rs:235-237), preserving all other fields when not provided.
  3. **Serializer parity** — ensure doc.rs frontmatter output is byte-identical to `wm_page`'s writer for the same inputs (AC-4 in wm-doc-fix-02 depends on this). If wm_page's serializer is reusable, call it; otherwise replicate its exact YAML shape. Leave full delegation (OQ-1) to planning of wm-doc-fix-02.
  4. **Tests (AC-1/2/3)** — add to apps/wm-core/tests/mcp_test.rs (or a doc-focused test) using the inproc harness (tests/helpers/inproc.rs, pattern at mcp_test.rs:217/249):
     - AC-1: create with `type: "spec"` → read file, frontmatter contains `type: spec`
     - AC-2: update with `type: "howto"` → frontmatter retyped, title + body preserved
     - AC-3: update with tags → tags persisted, existing type preserved when not provided
  5. **Validate** — `cargo build -p wm-core`; `cargo test --no-default-features --features "code-intel,lsp" -p wm-core --lib` + mcp_test suite; `cargo clippy -p wm-core -- -D warnings` (dead_code annotation must be gone).

  ### Related context
  - Approved spec: @wiki/specs/wm-doc-type-frontmatter (FR-1/2/3, NFR-1/2; AC-1/2/3)
  - GitHub issue #126 (root cause: dead field + missing field, masked by #[allow(dead_code)] and zero wm_doc frontmatter tests)
  - Adjacent draft spec @wiki/specs/retire-wm-doc (retire wm_doc, rename wm_page→wm_doc) — NOT in scope here; noted so consolidation planning can revisit this task's fix if that direction is ever approved
  - wm-doc-fix-02 covers parity tests + regression; wm-doc-fix-03 covers the repo-wide dead_code ban
time_spent: 0h 11m
---

Wire `type` through `wm_doc.create` (currently declared but ignored — `r#type: _` at apps/wm-core/src/mcp/tools/doc.rs:157, `build_markdown` at :187 never receives it) and add `type` + `tags` to `wm_doc.update` (schema at doc.rs:40-44 has neither). Persist exactly as `wm_page` does; when type absent, fall back to path-derived type. From spec wiki:specs:wm-doc-type-frontmatter (FR-1/2/3, NFR-1/2).