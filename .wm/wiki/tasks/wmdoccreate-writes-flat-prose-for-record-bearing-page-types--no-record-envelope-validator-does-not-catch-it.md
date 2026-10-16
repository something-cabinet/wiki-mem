---
title: wm_doc.create writes flat prose for record-bearing page types — no record envelope, validator does not catch it
type: task
id: "wiki:tasks:wmdoccreate-writes-flat-prose-for-record-bearing-page-types--no-record-envelope-validator-does-not-catch-it"
status: todo
priority: high
tags: [bug, tool-reliability, mcp, record-format, wm-doc]
acceptance_criteria:
  - text: "A record-bearing page created via wm_doc.create has the canonical record envelope for its type (test)"
  - text: "wm_validate rejects a record-bearing page whose body is not a canonical record (test)"
  - text: "No regression for non-record-bearing types (e.g. note) (test)"
implementation_notes: 'Second reproduction 2026-10-16: wm_page.create reproduces the same defect. wm_page.create({action:''create'', path:''specs/yaml-frontmatter-serializer-unification'', title:''...'', content:<prose>}) returned {id, path, type:''spec''} and wrote a body of flat prose with NO schema_version/state/questions/answers envelope (grep for ''^schema_version: 1'' found none). The create builder (page_create_builder_service.rs build_create_body) exists but the wm_page tool path did not apply it. Workaround used: hand-built envelope written directly, then wm_index.rebuild. Same for two task pages created this session.'
---

Repro 2026-10-16: wm_doc.create({action:'create', path:'specs/memory-write-decision-gate', type:'spec', content:<prose>}) returned success and persisted a page whose body was flat prose starting '## Overview' — no `schema_version: 1` / `state: |-` / `questions:` / `answers:` envelope. Every pre-existing spec page carries the full envelope (checked: remove-global-memory-layer, cross-entity-hybrid-search, wm-doc-typed-pages, eval-gliner-typed-decisions, typed-decision-doc-format). wm_validate_check({entity:'wiki:specs:memory-write-decision-gate'}) returned status pass with 0 errors on the flat body, so the missing envelope is silent. Sibling creation paths DO build the envelope: wm_decision.create uses page::build_record_body (mcp/tools/decision.rs:135) and page_create_builder_service.rs:45 calls build_record_body. Expectation: creating a record-bearing page type (spec, memory, task, decision, pattern, concept, howto, reference, rule, core) via wm_doc.create must emit the canonical envelope for that page type, OR the validator must reject a record-bearing page whose body is not a record. Manual workaround used: wm_page.update with a hand-built envelope + canonical spec question set.