---
id: wiki:specs:versions-system
title: Version History System
type: spec
status: draft
tags: [versions, history, knowns-parity]
---
id: wiki:specs:versions-system

## Overview

Add field-level version history for tasks and docs. Every update records which fields changed (old → new value) as a version entry. Versions stored as JSON files per entity in `.wm/versions/`. FSRS determines when old versions get compacted.

## Locked Decisions

- D11: Version history in scope
- D12: Field-level diffs (not full snapshots)
- D13: FSRS-driven compaction (low FSRS retention → collapse into gap)

## Requirements

### FR-1: Version storage
- One JSON file per entity: `.wm/versions/task-{id}.json`, `.wm/versions/doc-{safe-path}.json`
- Format: `{ "entityId": "...", "currentVersion": 5, "versions": [ ... ] }`

### FR-2: Version entry format
```rust
struct TaskVersion {
    id: String,        // "v1", "v2"
    version: u32,      // 1, 2
    timestamp: String, // ISO
    author: Option<String>,
    changes: Vec<FieldChange>,
    compacted: bool,   // true if this entry is a compaction gap
}

struct FieldChange {
    field: String,
    old_value: Option<serde_json::Value>,
    new_value: Option<serde_json::Value>,
}
```

### FR-3: When versions are created
- `wm_task.update` → `SaveVersion` before write
- `wm_task.check_ac` / `uncheck_ac` → `SaveVersion`
- `wm_page.update` → `SaveVersion` for doc pages
- On task/doc creation → initial version recording all initial fields

### FR-4: FSRS compaction
- After saving a new version, check if compaction is needed
- FSRS score for each version based on age: `R(t) = (1 + (t/days))^-1`
- Versions with `R(t) < threshold` (e.g., < 0.1) get collapsed into a single `{ changes: [], compacted: true }` gap entry
- Keep at least one full version per day for the last 7 days

### FR-5: Version retrieval
- `wm_version.list { entity_id: "task:abc123" }` → return version IDs + timestamps
- `wm_version.get { entity_id: "task:abc123", version: "v3" }` → return full diff for that version
- `wm_version.rollback { entity_id: "task:abc123", version: "v3" }` → restore entity state to that version

### FR-6: FSRS parameter reuse
- Reuse the `ScoringConfig.recency_stability_days` from existing config for FSRS half-life
- Default 7 days: a version from 7 days ago has R ≈ 0.5, from 30 days ≈ 0.1

## Implementation

### Files to create/modify
- `wm-core/src/version.rs` — VersionStore, TaskVersion, FieldChange types
- `wm-core/src/engine/mod.rs` — export version module
- `wm-core/src/mcp/tools/version.rs` — wm_version tool (list, get, rollback)
- `wm-core/src/mcp/tools/mod.rs` — register wm_version
- `wm-core/src/mcp/tools/task.rs` — call `VersionStore.save_version()` on update
- `wm-core/src/mcp/tools/page.rs` — call `VersionStore.save_version()` on update

### VersionStore API
```rust
pub struct VersionStore {
    root: PathBuf,
}

impl VersionStore {
    pub fn new(root: PathBuf) -> Self;
    pub fn save_task_version(&self, task_id: &str, changes: Vec<FieldChange>) -> Result<()>;
    pub fn save_doc_version(&self, doc_path: &str, changes: Vec<FieldChange>) -> Result<()>;
    pub fn get_task_history(&self, task_id: &str) -> Result<TaskVersionHistory>;
    pub fn get_doc_history(&self, doc_path: &str) -> Result<DocVersionHistory>;
    pub fn rollback_task(&self, task_id: &str, version: u32, engine: &EngineState) -> Result<()>;
}
```

## Acceptance Criteria

- [ ] AC-1: Updating a task creates a version entry with field-level diffs
- [ ] AC-2: Updating a doc creates a version entry
- [ ] AC-3: Version entries are readable through `wm_version.list` / `wm_version.get`
- [ ] AC-4: FSRS compaction collapses old versions after threshold
- [ ] AC-5: Rollback restores entity to specified version state
- [ ] AC-6: All existing tests pass
