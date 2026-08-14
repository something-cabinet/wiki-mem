---
title: Add single frontmatter builder choke point for all create paths
type: task
id: wiki:tasks:add-single-frontmatter-builder-choke-point-for-all-create-paths
status: done
priority: high
tags: [from-oracle, refactor, bugfix, linus-remediation]
parent: wiki:tasks:apply-oracle-recommendations-from-linus-critique-review
acceptance_criteria:
  - text: "One typed build_frontmatter(&[(key, Value)]) choke point in page/helpers serializing scalars through yaml_scalar and lists through render_yaml_value"
  - text: "All 7 string-built create paths routed through it: task create (task/mod.rs:329), task subtask (:729), page (page/mod.rs:154), doc (doc.rs:492), memory (memory.rs:310), decision (decision.rs:73), lint auto-fixer (lint.rs:35)"
  - text: "Line-based editors (set_yaml_field etc.) kept for update paths only"
  - text: "Round-trip test: create with title containing [ and : parses back correctly"
  - text: "cargo build + clippy -D warnings + mcp_test suite green"
implementation_notes: 'DONE. New page/helpers/frontmatter_value.rs (FrontmatterValue enum: Scalar/Id/Int/List/Nested) + build_frontmatter(&[(&''static str, FrontmatterValue)]) choke point in page/helpers/yaml_helper.rs — scalars via yaml_scalar (quotes only when needed), ids via yaml_quote (always double-quoted), lists inline flow, nested sub-mappings. Routed all string-built CREATE paths: task/mod.rs create + subtask (the subtask raw-title path was the genuine corruption vector), page/mod.rs, memory.rs, decision.rs, and the lint auto-fixer graph/lint.rs::auto_fix_missing_frontmatter (via early-continue, no-else). doc.rs skipped (now an alias over the page path). Line-based editors kept for UPDATE paths only. TDD: mcp_test task_create_with_yaml_breaking_title_round_trips (title "[BLOCK]: fix: the thing") RED->GREEN. Verified: clippy -D warnings clean; lib 157; mcp_test 54. ACs satisfied.'
---

From wiki:tasks:apply-oracle-recommendations-from-linus-critique-review AC-2. Oracle verdict LANDED on structure (grep claim rejected — keep the CI ban). 7 call sites build string frontmatter with inconsistent quoting: task create (task/mod.rs:329 — tags/assignee/parent/spec raw), task subtask (:729-734 — raw title/tags), page (page/mod.rs:154), doc (doc.rs:492-499 — raw tags), memory (memory.rs:310), decision (decision.rs:73), and the lint auto-fixer itself (lint.rs:35 — the repair tool can author corruption). yaml_helper adoption is opt-in per call site; there is no create-path choke point. Fix: one typed build_frontmatter in page/helpers that serializes every scalar through yaml_scalar and lists through render_yaml_value; route all seven create paths through it; keep line-based editors for updates only (byte-preservation justified — 652e07 comments correct). NOTE: keep the CI grep ban — allow_attributes_without_reason enforces reasons, not a ban.