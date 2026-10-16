---
title: Implement memory write decision gate — heuristic answering of the memory question set
type: task
id: "wiki:tasks:implement-memory-write-decision-gate--heuristic-answering-of-the-memory-question-set"
status: done
priority: high
tags: [from-spec, memory, decision-gate, heuristics, typed-decisions]
spec: wiki:specs:memory-write-decision-gate
acceptance_criteria:
  - text: "AC-1: near-duplicate add returns status skipped and writes no file (test)"
  - text: "AC-2: non-duplicate project add creates page with all four answers populated (test)"
  - text: "AC-3: response carries nearest{id,score} in both outcomes (test)"
  - text: "AC-4: session-layer add still inserts into session store and is unaffected (test)"
  - text: "AC-5: confidence=high only when content cites concrete anchors; medium otherwise (table test)"
  - text: "AC-6: overriding the threshold config changes skip behavior (test)"
  - text: "AC-7: dedup_action never emits merge/supersede in v1 (test)"
  - text: "AC-8: low is accepted as a confidence answer value by the record layer (test)"
  - text: "AC-9: typed-decision-doc-format D-H updated to state all types are record-bearing (doc check)"
  - text: "AC-10: wm_memory tool description documents the new skipped status"
relates_to:
  - {type: relates_to, target: wiki:specs:memory-write-decision-gate}
implementation_plan: |-
  ## Implementation Plan — Memory Write Decision Gate

  Owned files (write scope):
  - packages/wm-constants/src/defaults.rs (new const)
  - apps/wm-core/src/config/models/scoring_config_model.rs (tunable)
  - apps/wm-core/src/lib.rs (mod decl)
  - apps/wm-core/src/memory_gate/** (new module)
  - apps/wm-core/src/mcp/tools/memory.rs (wire gate + description)
  - apps/wm-core/tests/mcp_test.rs (tests)
  - .wm/wiki/specs/typed-decision-doc-format.md (AC-9 doc)

  ### Step 0 — Baseline: cargo test -p wm-core (confirm green).

  ### Step 1 — RED: tests first (mirror memory_add_emits_record_body :2625)
  - near-dup (exact slug exists, or top-1 memory score >= threshold) -> status skipped, NO file written
  - novel add -> page created; answers = layer=project, store_or_skip=true, dedup_action=new, confidence=high|medium
  - nearest{id,score} present in BOTH outcomes
  - session add unaffected
  - confidence table: anchors (wiki: id / file path / eval number) -> high; else medium
  - config override changes skip behaviour
  - dedup_action never emits merge|supersede
  - validator accepts low as a confidence value (record-layer reservation, D5 / AC-8)
  - tool description mentions skipped (via registry.list_tools())

  ### Step 2 — Constants + config
  - wm-constants/defaults.rs: DEFAULT_MEMORY_DEDUP_THRESHOLD
  - ScoringConfig: #[serde(default)] memory_dedup_threshold: f64 + default fn + impl Default + assertion in test_scoring_config_defaults

  ### Step 3 — New module apps/wm-core/src/memory_gate/
  - enums DedupAction {New, Skip}, Confidence {High, Medium} (+ reserved Low via a named predicate)
  - structs GateEvidence {nearest_id, nearest_score}, MemoryWriteDecision {store, dedup_action, layer, confidence, evidence}
  - fn evaluate_memory_write(engine, title, content, layer_hint) -> Result<MemoryWriteDecision, ToolError>
  - duplicate check: exact target path exists -> skip; else BM25 top-1 filtered to page_type == memory >= threshold -> skip
  - (query engine.bm25_index directly: run_unified_search does NOT filter by type; the memory index is a stub)
  - confidence: provenance-categorical (anchors -> high, else medium); low predicate reserved, named, not emitted
  - answers body: build_record_body(Memory, content) then parser::write_record_answers
  - DRY: reuse the existing slugify, do not duplicate it

  ### Step 4 — Wire into WmMemoryAction::Add (project branch only; session unchanged)
  - gate before create_page_through_builder; pass the answers-bearing record body as content (pass-through path)
  - skip -> status skipped + reason + nearest{id,score}
  - store -> existing response + nearest evidence
  - update the tool description literal (memory.rs:211) to document skipped

  ### Step 5 — AC-9: amend typed-decision-doc-format D-H (:26) re the memory/rule/core clause

  ### Step 6 — Verify: cargo fmt (file-scoped), cargo clippy -p wm-core (zero warnings), cargo test -p wm-core, wm_validate

  ### Risks
  - threshold default is uncalibrated -> measure the score distribution over the existing memory corpus to pick a conservative default; evidence always returned; D4 eval gate before lowering
  - no test precedent for runtime config override -> set the engine config RwLock in the test (new pattern, flag it)
  - write_record_answers takes full content -> integrate carefully with the create path

  ### Out of scope: merge/supersede (D2), wm_decision heuristic mode (D8), low->gate coupling (Open Question)
time_started: 2026-10-16T03:47:57.742887+00:00
time_spent: 1h 9m
implementation_notes: |-
  ## Implementation notes (orchestrator, 2026-10-16)

  Delivered: apps/wm-core/src/memory_gate/ (new) + wiring in mcp/tools/memory.rs; config tunable memory_dedup_threshold on ScoringConfig; DEFAULT_MEMORY_DEDUP_THRESHOLD in wm-constants; 18 memory-scoped tests in mcp_test.rs; AC-9 D-H amendment in typed-decision-doc-format.

  Verified by orchestrator (not the lane): cargo clippy -p wm-core clean (0 warnings); cargo test -p wm-core -> lib 253 pass, mcp_test 113 pass (all memory_gate tests), security 18, path_resolution 3. Only semantic_test.rs onnx::* fails (2) - pre-existing/environmental (embedding model 'none' not loaded); semantic_test.rs and search/query.rs are untouched by this change.

  AC PERSISTENCE CAVEAT: wm_task.check_ac returned success for all 10 but did NOT persist checked flags (file shows no checked:). This is the known defect @wiki/tasks:wmtask-checkac-and-status-updates-report-success-without-persisting. All ACs are nonetheless empirically verified by the test suite + the D-H doc check.

  Accepted debt: DEFAULT_MEMORY_DEDUP_THRESHOLD=100.0 is a first-pass calibration (raw score_doc top-1: exact-dup min 156.89, unrelated <=38). Per D4 an eval step is required before lowering it. Captured in memory wiki:memory:memory-dedup-threshold-calibration-bm25index-search-normalization-gotcha.

  Deviation from plan (sound): used raw Bm25Index::score_doc instead of Bm25Index::search because search normalizes top-1 to 1.0 (unusable as an absolute threshold).

  General wiki health check: 4 pre-existing errors, all unrelated (tasks missing acceptance criteria: missing-license-file..., shim-template-command-parity-guard, agent-handbooks..., agent-dx-freshness...).
---

Add a deterministic heuristic gate to wm_memory.add per @wiki/specs:memory-write-decision-gate. Peer answerer in wm-core; DecisionBackend/DecisionRuntime untouched. See spec for locked decisions D1-D8. NOTE: creating the spec revealed wm_doc.create does not emit a record envelope for record-bearing types (separate bug task filed).