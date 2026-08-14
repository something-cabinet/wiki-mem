---
title: dead_code ban enforcement
type: task
id: wiki:tasks:deadcode-ban-enforcement
status: done
priority: medium
tags:
- from-spec
- spec:wm-doc-type-frontmatter
- wm-doc-fix-03
spec: wiki:specs:wm-doc-type-frontmatter
acceptance_criteria:
- text: 'AC-6: No allow(dead_code) attribute remains in the repo (excluding target/); clippy -D warnings clean; http_daemon.rs converted to expect(dead_code)'
  checked: true
- text: 'AC-7: CI check step fails the build when an allow(dead_code) attribute is introduced'
---

Ban allow(dead_code) repo-wide (D3): remove the doc.rs annotation (part of wm-doc-fix-01), convert the 6 allow(dead_code, reason) annotations in apps/wm-core/tests/helpers/http_daemon.rs to expect(dead_code) (self-cleaning, needs rustc 1.81+ — confirm MSRV), and add a deterministic CI grep check to .github/workflows/ci.yml check job that fails when allow(dead_code) appears (excluding target/). Clippy verified: no attribute-ban lint exists (clippy::disallowed_attrs unavailable; allow_attributes_without_reason only forces reasons). From spec wiki:specs:wm-doc-type-frontmatter (FR-5/6, NFR-3, AC-6/7).