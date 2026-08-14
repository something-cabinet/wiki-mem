---
title: wm_page parity plus regression coverage
type: task
id: wiki:tasks:wmpage-parity-plus-regression-coverage
status: done
priority: medium
tags:
- from-spec
- spec:wm-doc-type-frontmatter
- wm-doc-fix-02
spec: wiki:specs:wm-doc-type-frontmatter
acceptance_criteria:
- text: 'AC-4: Parity test — wm_doc.create and wm_page.create with identical inputs produce byte-identical frontmatter for type'
  checked: true
- text: 'AC-5: Existing wm_page, wm_doc, and MCP suite tests pass unchanged'
---

Add parity + regression tests: identical inputs through wm_doc.create and wm_page.create must produce byte-identical frontmatter (type included); all existing wm_page/wm_doc/MCP suites must pass unchanged. Follow the inproc harness pattern (tests/helpers/inproc.rs) like mcp_test.rs:217/249. From spec wiki:specs:wm-doc-type-frontmatter (FR-4, AC-4/5).