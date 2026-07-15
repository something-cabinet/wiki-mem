---
title: Status Model Rework
type: spec
status: draft
tags: [status, models, cdd, enum-page, knowns-parity]
---

## Overview

Replace WM's monolithic `PageStatus` and flat `WikiPageMeta` with per-type validated statuses, an `enum Page` dispatch model, and CDD-compliant types throughout. Covers model enrichment (Knowns parity gaps), status validation, and fixing compile-time safety violations.

## Locked Decisions

- D1: Keep `PageStatus` as a single enum. No split into per-type enums.
- D2: `PageType::allowed_statuses()` for per-type validation at tool layer.
- D3: Use `pub const` for status strings instead of match arms.
- D4: `MemoryStatus` enum: `Active, Stale, Archived`.
- D5: `published: bool` on `WikiPageMeta`.
- D6: Memory stays outside the graph (separate struct, separate JSON files).
- D7: Spec/fulfills linkage uses `relates_to` typed edges, not frontmatter fields.
- D8: `time_entries: Vec<TimeEntry>` in task frontmatter for history. Keep single active timer.
- D9: Supersedence uses `relates_to` typed edges (`supersedes`), not frontmatter.
- D10: `consequences: Option<String>` on `DecisionData`.
- D11: Per-type `XxxData` wrapper structs with unified naming (`TaskData`, `SpecData`, `DecisionData`, `PatternData`).
- D12: `enum Page` dispatch over `Option<XxxData>` on a flat struct.

## Requirements

### FR-1: enum Page model

Replace the flat `WikiPageMeta` struct with an `enum Page` where each page type is a variant containing shared metadata + typed per-type data.

Shared metadata struct:

```
pub struct WikiPageMeta {
    pub id: String, pub title: String, pub tags: Vec<String>,
    pub status: PageStatus, pub published: bool,
    pub confidence: Option<Confidence>, pub aliases: Vec<String>,
    pub superseded_by: Option<String>, pub version: Option<String>,
    pub sources: Vec<String>, pub relates_to: Vec<(EdgeType, String)>,
    pub parent: Option<String>,
    pub path: PathBuf, pub created_at: String, pub updated_at: String,
}
```

Per-type data structs:

```
pub struct TaskData {
    pub acceptance_criteria: Vec<AcceptanceCriterion>,
    pub estimate: Option<u32>, pub prerequisites: Vec<String>,
    pub difficulty: Option<String>,
    pub time_spent: Option<String>, pub time_entries: Vec<TimeEntry>,
}

pub struct SpecData {
    pub functional_requirements: Vec<FunctionalRequirement>,
    pub non_functional_requirements: Vec<NonFunctionalRequirement>,
    pub general_goals: Vec<GeneralGoal>,
    pub stakeholders: Vec<String>,
}

pub struct DecisionData {
    pub context: String, pub options: Vec<String>,
    pub rationale: String, pub outcome: String,
    pub consequences: Option<String>,
}

pub struct PatternData {
    pub problem: String, pub solution: String, pub consequences: String,
}
```

Page enum:

```
pub enum Page {
    Task     { meta: WikiPageMeta, data: TaskData },
    Spec     { meta: WikiPageMeta, data: SpecData },
    Decision { meta: WikiPageMeta, data: DecisionData },
    Pattern  { meta: WikiPageMeta, data: PatternData },
    Concept  { meta: WikiPageMeta },
    HowTo    { meta: WikiPageMeta },
    Reference{ meta: WikiPageMeta },
}
```

### FR-2: Page::meta() accessor

```
impl Page {
    pub fn meta(&self) -> &WikiPageMeta {
        match self {
            Task { meta, .. } | Spec { meta, .. } | Decision { meta, .. }
            | Pattern { meta, .. } | Concept { meta } | HowTo { meta }
            | Reference { meta } => meta,
        }
    }
}
```

All shared-field access goes through `page.meta().id`, `page.meta().title`, etc.

### FR-3: PageType::allowed_statuses()

```
impl PageType {
    pub fn allowed_statuses(&self) -> &[PageStatus] {
        match self {
            PageType::Task => &[Todo, InProgress, InReview, Done, Blocked, Cancelled],
            PageType::Spec => &[Draft, Reviewed, Approved, Superseded],
            PageType::Decision => &[Draft, Approved, Superseded, Rejected, Archived],
            _ => &[Draft, Reviewed, Approved, Archived],
        }
    }
}
```

Validate at tool entry points: `wm_page.create`, `wm_page.update`, `wm_task.create`, `wm_task.update`, `wm_decision.create`. Return `ToolError::invalid_params()` for disallowed statuses.

### FR-4: PageStatus constants

```
pub const TODO: &str = "todo";
pub const IN_PROGRESS: &str = "in-progress";
pub const IN_REVIEW: &str = "in-review";
pub const DONE: &str = "done";
pub const BLOCKED: &str = "blocked";
pub const CANCELLED: &str = "cancelled";
pub const DRAFT: &str = "draft";
pub const REVIEWED: &str = "reviewed";
pub const SUPERSEDED: &str = "superseded";
pub const APPROVED: &str = "approved";
pub const ACCEPTED: &str = "accepted";
pub const REJECTED: &str = "rejected";
pub const ARCHIVED: &str = "archived";
pub const ACTIVE: &str = "active";
pub const STALE: &str = "stale";
```

Method `as_str() -> &'static str` returns the constant. Must maintain backward compatibility with existing kebab-case frontmatter strings.

### FR-5: MemoryStatus enum

```
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum MemoryStatus { Active, Stale, Archived }
```

Add `status: Option<MemoryStatus>` to `MemoryEntry`. `None` treated as `Active` for backward compat. Update `wm_memory.list` to accept `status` filter.

### FR-6: CDD fix — PageType::as_str()

Replace all `format!("{:?}", meta.page_type).to_lowercase()` and `format!("{:?}", meta.status).to_lowercase()` with proper `as_str()` methods. `PageType` gets an `as_str()` method matching `PageStatus::as_str()`. Fixes the `Note` vs `note` Debug mismatch and the `InProgress` vs `in-progress` bug.

Affected files: page.rs, search/query.rs, graph.rs, mcp/tools/graph.rs, mcp/tools/project.rs, mcp/tools/search.rs, mcp/tools/task.rs (9 sites total).

### FR-7: CDD fix — relates_to typed edges

Change `relates_to: Vec<String>` to `Vec<(EdgeType, String)>` with a custom YAML deserializer. The `Relation` struct already exists in parser.rs for this purpose. Eliminates the fragile `split_once(':')` runtime parsing.

### FR-8: CDD fix — config strings to enums

Replace `PermissionsConfig.preset: String` with `enum PermissionPreset { ReadWrite, ReadOnly }`. Replace `SearchConfig.default_mode: String` with `SearchMode` (already exists as type). Replace `ScoringConfig.recency_model: String` with `enum RecencyModel { Fsrs, Linear, Exponential, None }`.

### FR-9: CDD fix — tool input typed enums

Change all `status: Option<String>`, `r#type: Option<String>`, `priority: Option<String>`, `mode: Option<String>`, `layer: Option<String>` in MCP tool input structs to proper enum types:

- `status` -> `Option<PageStatus>`
- `r#type` -> `Option<PageType>`
- `priority` -> `Option<Priority>`
- `mode` -> `Option<SearchMode>`
- `layer` -> `Option<MemoryLayer>` (new enum)

Serde `rename_all = "kebab-case"` handles the string to enum conversion. Bad values error at deserialization time instead of silently defaulting.

### FR-10: CDD fix — remove serde_json::Value round-trip in page update

Replace the `WmPageUpdateInput -> serde_json::Value -> page::update_page()` pattern with a direct `PageUpdateParams` struct passed through the type chain.

### NFR-1: Backward compatibility

- All existing wiki pages with current status values must parse without warnings
- Existing memory JSON files without `status` field parse as `None` (-> Active)
- Existing config files with string values (`"hybrid"`, `"fsrs"`, `"read-write"`) parse into new enums via serde rename
- `accepted` status maps to `PageStatus::Approved`

### NFR-2: Petgraph unchanged

- Graph still uses `StableGraph<WikiPageMeta, EdgeType>` under the hood via conversion
- Memory stays as separate JSON files, not in graph
- No database migration needed

## Acceptance Criteria

- [ ] AC-1: `PageType::as_str()` exists and produces kebab-case output
- [ ] AC-2: No `format!("{:?}", ...)` on page_type or status anywhere in the codebase
- [ ] AC-3: `Page::meta()` accessor compiles and works on all variants
- [ ] AC-4: `wm_task.update` with `status: approved` returns an error
- [ ] AC-5: `wm_decision.create` with `status: in-progress` returns an error
- [ ] AC-6: `wm_page.create` with `status: todo` on a concept page returns an error
- [ ] AC-7: All existing YAML frontmatter with current status values parses without warnings
- [ ] AC-8: Old memory JSON files without `status` field parse as `None`
- [ ] AC-9: Config file with `"hybrid"`, `"fsrs"`, `"read-write"` values parses into new enums
- [ ] AC-10: `wm_memory.list` accepts a `status` filter parameter
- [ ] AC-11: `relates_to: Vec<(EdgeType, String)>` serializes/deserializes to/from YAML correctly
- [ ] AC-12: All MCP tool input structs use typed enums, not `Option<String>`
- [ ] AC-13: All existing tests pass
