---
title: Execute retire-wm-doc consolidation
type: task
id: wiki:tasks:execute-retire-wm-doc-consolidation
status: done
priority: medium
tags:
- from-oracle
- refactor
- consolidation
- linus-remediation
parent: wiki:tasks:apply-oracle-recommendations-from-linus-critique-review
acceptance_criteria:
- text: wm_doc becomes deprecated alias over wm_page path (same writer, no byte-imitation)
- text: doc.rs deleted or reduced to alias (parse_frontmatter duplicate doc.rs:459-486 removed)
- text: Parity tests deleted (mcp_test.rs:407-429)
- text: ~650 lines removed; retire-wm-doc spec ACs 1-9 met
- text: cargo build + clippy + mcp_test suite green
implementation_notes: 'Wave 1 review gate: GO-with-findings. Single-writer confirmed (byte-imitation build_markdown + duplicate parse_frontmatter deleted; wm_doc routes through page::handle_action); path confinement preserved (confine_doc_path); no dead code. P1 (silent output/filter contract change) resolved by making it EXPLICIT in the doc.rs module rustdoc — output now matches wm_page (id/sections/pages) and list filters by page-type name; this is intended convergence per retire-wm-doc, and re-adding a shape-imitation layer would violate no-compensating-layers. P2 write-action output-shape doc tracked in wiki:tasks:core-server-graph-twin-parity-p2s-and-wmdoc-write-action-output-docs. Verified: mcp_test 54; security_test 18; lib 157; clippy clean. ACs satisfied (~587 lines removed).'
---

From wiki:tasks:apply-oracle-recommendations-from-linus-critique-review AC-4. Oracle verdict LANDED: doc.rs carries its own parse_frontmatter (doc.rs:459-486) duplicating crate::parser which the same file uses at :253; build_markdown (doc.rs:488-499) exists solely to byte-imitate wm_page's string-built output — faithfully reproducing the inline-flow tags quirk and re-introducing the unquoted-tags bug (:497). Parity tests (mcp_test.rs:407-429) pin byte-identity of two writers — an artifact making the already-drafted retire-wm-doc spec harder to execute. Fix: execute the existing draft spec wiki:specs:retire-wm-doc — wm_doc becomes a deprecated alias calling the wm_page path (parity holds by construction), then delete doc.rs + parity tests. Note: this supersedes the wm-doc-fix parity approach from issue #126 wave (keep the type/tags fix, drop the two-writer duplication).