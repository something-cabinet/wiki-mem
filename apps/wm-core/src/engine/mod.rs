//! Core engine types — data structures used across the entire wiki memory engine.
//!
//! This module defines all domain types ([`EdgeType`], [`PageType`], [`WikiPageMeta`],
//! [`MemoryEntry`], etc.) and re-exports key types from submodules:
//! - [`EngineState`] — central runtime state
//! - [`WriteChannel`] — sequential file I/O
//! - [`IndexScheduler`] — debounced rebuild scheduler
//! - [`MainEngine`] — bootstrap and lifecycle

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
pub use crate::status::{Confidence, MemoryStatus, PageStatus, Priority};

pub mod write_channel;
pub mod scheduler;
pub mod state;
pub mod main;

pub use state::EngineState;
pub use write_channel::{WriteChannel, WriteOp};
pub use scheduler::IndexScheduler;
pub use main::MainEngine;

// ─── Edge Types ─────────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "kebab-case")]
pub enum EdgeType {
    Extends,
    Implements,
    ExampleOf,
    PartOf,
    RelatesTo,
    Supports,
    Contradicts,
    Supersedes,
    DependsOn,
    RequiredBy,
    Questions,
    Answers,
    References,
    SimilarTo,
    Causes,
    Mitigates,
    Custom(String),
}

impl EdgeType {
    pub fn priority(&self) -> u8 {
        match self {
            EdgeType::Extends => 10,
            EdgeType::Implements => 9,
            EdgeType::PartOf => 8,
            EdgeType::Supports => 7,
            EdgeType::ExampleOf => 6,
            EdgeType::DependsOn | EdgeType::RequiredBy => 5,
            EdgeType::Mitigates | EdgeType::Causes => 4,
            EdgeType::Contradicts | EdgeType::Questions => 3,
            EdgeType::Answers => 2,
            EdgeType::References | EdgeType::SimilarTo => 1,
            EdgeType::RelatesTo | EdgeType::Custom(_) => 0,
            EdgeType::Supersedes => 8,
        }
    }
}

// ─── Page Types ─────────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum PageType {
    Task,
    Spec,
    Concept,
    Pattern,
    Decision,
    Memory,
    Howto,
    Reference,
    #[serde(rename = "note")]
    Note,
}

impl PageType {
    /// Return the canonical string representation (kebab-case).
    pub fn as_str(&self) -> &'static str {
        match self {
            PageType::Task => "task",
            PageType::Spec => "spec",
            PageType::Concept => "concept",
            PageType::Pattern => "pattern",
            PageType::Decision => "decision",
            PageType::Memory => "memory",
            PageType::Howto => "howto",
            PageType::Reference => "reference",
            PageType::Note => "note",
        }
    }

    /// Return the list of allowed statuses for this page type.
    pub fn allowed_statuses(&self) -> &[PageStatus] {
        use PageStatus::*;
        match self {
            PageType::Task => &[Todo, InProgress, InReview, Done, Blocked, Cancelled],
            PageType::Spec => &[Draft, Reviewed, Approved, Superseded],
            PageType::Decision => &[Draft, Approved, Superseded, Rejected, Archived],
            _ => &[Draft, Reviewed, Approved, Archived],
        }
    }

    /// Priority rank for sorting (higher = more important).
    /// Note rank 0 — informal content never outranks deliberate types.
    pub fn priority_rank(&self) -> u8 {
        match self {
            PageType::Task => 7,
            PageType::Spec => 6,
            PageType::Pattern => 5,
            PageType::Concept => 4,
            PageType::Decision => 3,
            PageType::Memory => 2,
            PageType::Howto => 2,
            PageType::Reference => 1,
            PageType::Note => 0,
        }
    }
}

// ─── Memory Layers ──────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum MemoryLayer {
    Project,
    Global,
    Session,
}

impl Default for MemoryLayer {
    fn default() -> Self { MemoryLayer::Project }
}

impl MemoryLayer {
    /// Return the canonical string representation (kebab-case).
    pub fn as_str(&self) -> &'static str {
        match self {
            MemoryLayer::Project => "project",
            MemoryLayer::Global => "global",
            MemoryLayer::Session => "session",
        }
    }
}

// ─── Memory Data (for Page::Memory variant) ────────────

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct MemoryData {
    pub layer: MemoryLayer,
    pub ttl_days: Option<u32>,
    pub last_verified: Option<String>,
    pub merged_into: Option<String>,
    pub rejected_reason: Option<String>,
    pub metadata: std::collections::HashMap<String, String>,
}

// ─── Memory Entry (deprecated — kept for session memory only) ──

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MemoryEntry {
    pub id: String,
    pub title: String,
    pub content: String,
    pub tags: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
    #[serde(default)]
    pub status: Option<MemoryStatus>,
}

// ─── Spec-specific fields ───────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FunctionalRequirement {
    pub id: String,
    pub description: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NonFunctionalRequirement {
    pub id: String,
    pub description: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GeneralGoal {
    pub description: String,
}

// ─── Acceptance Criterion ───────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AcceptanceCriterion {
    pub text: String,
    pub checked: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TimeEntry {
    pub started_at: String,
    pub ended_at: Option<String>,
    pub duration_s: Option<u64>,
    pub note: Option<String>,
}

// ─── Per-type data structs ──────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TaskData {
    pub acceptance_criteria: Vec<AcceptanceCriterion>,
    pub estimate: Option<u32>,
    pub prerequisites: Vec<String>,
    pub difficulty: Option<String>,
    pub time_spent: Option<String>,
    pub time_entries: Vec<TimeEntry>,
    pub implementation_plan: Option<String>,
    pub implementation_notes: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SpecData {
    pub functional_requirements: Vec<FunctionalRequirement>,
    pub non_functional_requirements: Vec<NonFunctionalRequirement>,
    pub general_goals: Vec<GeneralGoal>,
    pub stakeholders: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DecisionData {
    pub context: String,
    pub options: Vec<String>,
    pub rationale: String,
    pub outcome: String,
    pub consequences: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PatternData {
    pub when_to_use: String,
    pub example: String,
}

// ─── Serde helpers for relates_to ──────────────────────────

/// Custom serde module for `Vec<(EdgeType, String)>` that serializes
/// as `[{type: extends, target: "wiki:..."}]` in YAML.
mod relates_to_vec {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use super::EdgeType;

    #[derive(Serialize, Deserialize)]
    struct Relation {
        #[serde(rename = "type")]
        edge_type: String,
        target: String,
    }

    pub fn serialize<S>(val: &[(EdgeType, String)], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let items: Vec<Relation> = val
            .iter()
            .map(|(et, target)| Relation {
                edge_type: edge_type_to_yaml_str(et),
                target: target.clone(),
            })
            .collect();
        items.serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<(EdgeType, String)>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let items = Vec::<Relation>::deserialize(deserializer)?;
        Ok(items
            .into_iter()
            .map(|r| (super::parse_edge_type_flexible(&r.edge_type), r.target))
            .collect())
    }

    fn edge_type_to_yaml_str(et: &EdgeType) -> String {
        match et {
            EdgeType::Extends => "extends".into(),
            EdgeType::Implements => "implements".into(),
            EdgeType::ExampleOf => "example_of".into(),
            EdgeType::PartOf => "part_of".into(),
            EdgeType::RelatesTo => "relates_to".into(),
            EdgeType::Supports => "supports".into(),
            EdgeType::Contradicts => "contradicts".into(),
            EdgeType::Supersedes => "supersedes".into(),
            EdgeType::DependsOn => "depends_on".into(),
            EdgeType::RequiredBy => "required_by".into(),
            EdgeType::Questions => "questions".into(),
            EdgeType::Answers => "answers".into(),
            EdgeType::References => "references".into(),
            EdgeType::SimilarTo => "similar_to".into(),
            EdgeType::Causes => "causes".into(),
            EdgeType::Mitigates => "mitigates".into(),
            EdgeType::Custom(s) => s.clone(),
        }
    }
}

/// Parse an edge type string flexibly (supports multiple aliases).
pub(crate) fn parse_edge_type_flexible(s: &str) -> EdgeType {
    match s.to_lowercase().as_str() {
        "extends" => EdgeType::Extends,
        "implements" => EdgeType::Implements,
        "example_of" | "exampleof" | "example-of" => EdgeType::ExampleOf,
        "part_of" | "partof" | "part-of" => EdgeType::PartOf,
        "relates_to" | "relates-to" | "relatesto" | "related" => EdgeType::RelatesTo,
        "supports" => EdgeType::Supports,
        "contradicts" => EdgeType::Contradicts,
        "supersedes" => EdgeType::Supersedes,
        "depends_on" | "dependson" | "depends-on" => EdgeType::DependsOn,
        "required_by" | "requiredby" | "required-by" => EdgeType::RequiredBy,
        "questions" => EdgeType::Questions,
        "answers" => EdgeType::Answers,
        "references" => EdgeType::References,
        "similar_to" | "similarto" | "similar-to" | "similar" => EdgeType::SimilarTo,
        "causes" => EdgeType::Causes,
        "mitigates" => EdgeType::Mitigates,
        custom => EdgeType::Custom(custom.to_string()),
    }
}

// ─── Wiki Page Metadata ─────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WikiPageMeta {
    pub id: String,
    pub title: String,
    pub tags: Vec<String>,
    pub status: PageStatus,
    pub published: bool,
    pub priority: Option<Priority>,
    pub confidence: Option<Confidence>,
    pub assignee: Option<String>,
    pub aliases: Vec<String>,
    pub superseded_by: Option<String>,
    pub version: Option<String>,
    pub sources: Vec<String>,
    pub parent: Option<String>,
    #[serde(with = "relates_to_vec")]
    pub relates_to: Vec<(EdgeType, String)>,
    pub path: PathBuf,
    pub created_at: String,
    pub updated_at: String,
    pub page_type: PageType,
    pub order: Option<i32>,
    // Per-type data — stored as Option for graph compatibility
    pub task_data: Option<TaskData>,
    pub spec_data: Option<SpecData>,
    pub decision_data: Option<DecisionData>,
    pub pattern_data: Option<PatternData>,
    pub memory_data: Option<MemoryData>,
}

// ─── Typed Page Enum (public API) ───────────────────────────

#[derive(Clone, Debug)]
pub enum Page {
    Task { meta: WikiPageMeta, data: TaskData },
    Spec { meta: WikiPageMeta, data: SpecData },
    Decision { meta: WikiPageMeta, data: DecisionData },
    Pattern { meta: WikiPageMeta, data: PatternData },
    Memory { meta: WikiPageMeta, data: MemoryData },
    Concept { meta: WikiPageMeta },
    HowTo { meta: WikiPageMeta },
    Note { meta: WikiPageMeta },
    Reference { meta: WikiPageMeta },
}

impl Page {
    pub fn meta(&self) -> &WikiPageMeta {
        match self {
            Page::Task { meta, .. } | Page::Spec { meta, .. }
            | Page::Decision { meta, .. } | Page::Pattern { meta, .. }
            | Page::Memory { meta, .. } | Page::Concept { meta }
            | Page::HowTo { meta } | Page::Note { meta }
            | Page::Reference { meta } => meta,
        }
    }

    pub fn meta_mut(&mut self) -> &mut WikiPageMeta {
        match self {
            Page::Task { meta, .. } | Page::Spec { meta, .. }
            | Page::Decision { meta, .. } | Page::Pattern { meta, .. }
            | Page::Memory { meta, .. } | Page::Concept { meta }
            | Page::HowTo { meta } | Page::Note { meta }
            | Page::Reference { meta } => meta,
        }
    }

    pub fn page_type(&self) -> PageType {
        match self {
            Page::Task { .. } => PageType::Task,
            Page::Spec { .. } => PageType::Spec,
            Page::Decision { .. } => PageType::Decision,
            Page::Pattern { .. } => PageType::Pattern,
            Page::Memory { .. } => PageType::Memory,
            Page::Concept { .. } => PageType::Concept,
            Page::HowTo { .. } => PageType::Howto,
            Page::Note { .. } => PageType::Note,
            Page::Reference { .. } => PageType::Reference,
        }
    }
}

/// Convert a WikiPageMeta (with Option data) into the typed Page enum.
/// Panics if the data doesn't match the page_type.
impl From<WikiPageMeta> for Page {
    fn from(mut wpm: WikiPageMeta) -> Self {
        let pt = wpm.page_type.clone();
        match pt {
            PageType::Task => Page::Task {
                data: wpm.task_data.take().expect("TaskData missing for Task page"),
                meta: wpm,
            },
            PageType::Spec => Page::Spec {
                data: wpm.spec_data.take().expect("SpecData missing for Spec page"),
                meta: wpm,
            },
            PageType::Decision => Page::Decision {
                data: wpm.decision_data.take().expect("DecisionData missing for Decision page"),
                meta: wpm,
            },
            PageType::Pattern => Page::Pattern {
                data: wpm.pattern_data.take().expect("PatternData missing for Pattern page"),
                meta: wpm,
            },
            PageType::Memory => Page::Memory {
                data: wpm.memory_data.take().expect("MemoryData missing for Memory page"),
                meta: wpm,
            },
            PageType::Concept => Page::Concept { meta: wpm },
            PageType::Howto => Page::HowTo { meta: wpm },
            PageType::Note => Page::Note { meta: wpm },
            PageType::Reference => Page::Reference { meta: wpm },
        }
    }
}

impl From<Page> for WikiPageMeta {
    fn from(page: Page) -> Self {
        let (page_type, mut meta) = match page {
            Page::Task { mut meta, data } => {
                meta.task_data = Some(data);
                (PageType::Task, meta)
            }
            Page::Spec { mut meta, data } => {
                meta.spec_data = Some(data);
                (PageType::Spec, meta)
            }
            Page::Decision { mut meta, data } => {
                meta.decision_data = Some(data);
                (PageType::Decision, meta)
            }
            Page::Pattern { mut meta, data } => {
                meta.pattern_data = Some(data);
                (PageType::Pattern, meta)
            }
            Page::Memory { mut meta, data } => {
                meta.memory_data = Some(data);
                (PageType::Memory, meta)
            }
            Page::Concept { meta } => (PageType::Concept, meta),
            Page::HowTo { meta } => (PageType::Howto, meta),
            Page::Note { meta } => (PageType::Note, meta),
            Page::Reference { meta } => (PageType::Reference, meta),
        };
        meta.page_type = page_type;
        meta
    }
}

// ─── Wiki Page Content ──────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WikiPageContent {
    pub raw: String,
    pub sections: Vec<SectionDoc>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SectionDoc {
    pub section_id: String,
    pub page_id: String,
    pub header: String,
    pub body: String,
}

// ─── Source Entry ───────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum SourceState {
    Pending,
    Processing,
    Done,
    Error,
    Stale,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SourceEntry {
    pub id: String,
    pub original_path: Option<String>,
    pub stored_path: PathBuf,
    pub content_hash: String,
    pub state: SourceState,
    pub added_at: String,
    pub last_processed_at: Option<String>,
    pub page_refs: Vec<String>,
    pub page_count: usize,
    pub error_message: Option<String>,
    pub retry_count: usize,
}

// ─── Audit Event ────────────────────────────────────────────

#[derive(Clone, Debug, Serialize)]
pub struct AuditEvent {
    pub timestamp: String,
    pub tool_name: String,
    pub action: String,
    pub duration_ms: i64,
    pub result: String,
    pub error_message: Option<String>,
    pub entity_refs: Vec<String>,
}

// ─── Graph Snapshot ─────────────────────────────────────────

pub type GraphSnapshot = (
    petgraph::stable_graph::StableGraph<WikiPageMeta, EdgeType>,
    HashMap<String, petgraph::stable_graph::NodeIndex>,
);

// ─── Template Prompt System ──────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TemplatePrompt {
    pub name: String,
    pub r#type: String,      // "text", "select", "confirm", "multiselect"
    pub message: String,
    pub initial: Option<serde_json::Value>,
    pub validate: Option<String>,
    pub choices: Option<Vec<String>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TemplateAction {
    pub r#type: String,       // "add", "addMany", "modify", "append"
    pub template: Option<String>,
    pub path: String,
    pub source: Option<String>,
    pub skip_if_exists: Option<bool>,
    pub when: Option<String>,
    pub insert_after: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TemplateConfig {
    pub name: String,
    pub description: String,
    pub doc: Option<String>,
    pub destination: Option<String>,
    pub prompts: Vec<TemplatePrompt>,
    pub actions: Vec<TemplateAction>,
    pub messages: Option<serde_json::Value>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_page_type_priority_rank() {
        assert_eq!(PageType::Task.priority_rank(), 7);
        assert_eq!(PageType::Spec.priority_rank(), 6);
        assert_eq!(PageType::Pattern.priority_rank(), 5);
        assert_eq!(PageType::Concept.priority_rank(), 4);
        assert_eq!(PageType::Decision.priority_rank(), 3);
        assert_eq!(PageType::Memory.priority_rank(), 2);
        assert_eq!(PageType::Howto.priority_rank(), 2);
        assert_eq!(PageType::Reference.priority_rank(), 1);
    }

    #[test]
    fn test_edge_type_priority() {
        assert!(EdgeType::Extends.priority() > EdgeType::RelatesTo.priority());
        assert!(EdgeType::Implements.priority() > EdgeType::References.priority());
        assert_eq!(EdgeType::DependsOn.priority(), EdgeType::RequiredBy.priority());
    }

    #[test]
    #[should_panic(expected = "TaskData missing for Task page")]
    fn test_page_from_missing_task_data() {
        let wpm = WikiPageMeta {
            id: "test".into(), title: "T".into(), tags: vec![], status: PageStatus::Todo,
            published: false, priority: None, confidence: None, assignee: None,
            aliases: vec![], superseded_by: None, version: None, sources: vec![],
            parent: None, relates_to: vec![], path: PathBuf::new(),
            created_at: "".into(), updated_at: "".into(), page_type: PageType::Task,
            order: None, task_data: None, spec_data: None, decision_data: None,
            pattern_data: None, memory_data: None,
        };
        let _page: Page = wpm.into();
    }

    #[test]
    fn test_allowed_statuses_task() {
        assert_eq!(PageType::Task.allowed_statuses(), &[
            PageStatus::Todo, PageStatus::InProgress, PageStatus::InReview,
            PageStatus::Done, PageStatus::Blocked, PageStatus::Cancelled,
        ]);
    }

    #[test]
    fn test_allowed_statuses_spec() {
        assert_eq!(PageType::Spec.allowed_statuses(), &[
            PageStatus::Draft, PageStatus::Reviewed, PageStatus::Approved, PageStatus::Superseded,
        ]);
    }

    #[test]
    fn test_allowed_statuses_decision() {
        assert_eq!(PageType::Decision.allowed_statuses(), &[
            PageStatus::Draft, PageStatus::Approved, PageStatus::Superseded,
            PageStatus::Rejected, PageStatus::Archived,
        ]);
    }

    #[test]
    fn test_allowed_statuses_content_types() {
        for pt in &[PageType::Concept, PageType::Pattern, PageType::Memory,
                    PageType::Howto, PageType::Reference, PageType::Note] {
            assert_eq!(pt.allowed_statuses(), &[
                PageStatus::Draft, PageStatus::Reviewed, PageStatus::Approved, PageStatus::Archived,
            ], "failed for {:?}", pt);
        }
    }

    #[test]
    fn test_page_type_as_str() {
        assert_eq!(PageType::Task.as_str(), "task");
        assert_eq!(PageType::Spec.as_str(), "spec");
        assert_eq!(PageType::Concept.as_str(), "concept");
        assert_eq!(PageType::Pattern.as_str(), "pattern");
        assert_eq!(PageType::Decision.as_str(), "decision");
        assert_eq!(PageType::Memory.as_str(), "memory");
        assert_eq!(PageType::Howto.as_str(), "howto");
        assert_eq!(PageType::Reference.as_str(), "reference");
        assert_eq!(PageType::Note.as_str(), "note");
    }

    #[test]
    fn test_page_meta_mut_roundtrip() {
        let mut page = Page::Concept {
            meta: WikiPageMeta {
                id: "test".into(), title: "Original".into(), tags: vec![],
                status: PageStatus::Draft, published: false, priority: None,
                confidence: None, assignee: None, aliases: vec![],
                superseded_by: None, version: None, sources: vec![],
                parent: None, relates_to: vec![], path: PathBuf::new(),
                created_at: "".into(), updated_at: "".into(), page_type: PageType::Concept,
                order: None, task_data: None, spec_data: None, decision_data: None,
                pattern_data: None, memory_data: None,
            },
        };
        page.meta_mut().title = "Updated".to_string();
        assert_eq!(page.meta().title, "Updated");
    }

    #[test]
    fn test_memory_layer_as_str() {
        assert_eq!(MemoryLayer::Project.as_str(), "project");
        assert_eq!(MemoryLayer::Global.as_str(), "global");
        assert_eq!(MemoryLayer::Session.as_str(), "session");
    }

    #[test]
    fn test_memory_layer_default() {
        assert_eq!(MemoryLayer::default(), MemoryLayer::Project);
    }

    #[test]
    fn test_edge_type_custom_roundtrip() {
        let custom = EdgeType::Custom("my-edge".into());
        let json = serde_json::to_string(&custom).unwrap();
        let deserialized: EdgeType = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, custom);
    }
}
