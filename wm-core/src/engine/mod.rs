//! Core engine types — data structures used across the entire wiki memory engine.
//!
//! This module defines all domain types ([`EdgeType`], [`PageType`], [`WikiPageMeta`],
//! [`MemoryEntry`], etc.) and re-exports key types from submodules:
//! - [`EngineState`] — central runtime state
//! - [`WriteChannel`] — sequential file I/O
//! - [`IndexScheduler`] — debounced rebuild scheduler
//! - [`MainEngine`] — bootstrap and lifecycle

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
pub use crate::status::{Confidence, PageStatus, Priority};

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

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum PageType {
    Task,
    Spec,
    Concept,
    Pattern,
    Decision,
    Howto,
    Reference,
    #[serde(rename = "note")]
    Note,
}

impl PageType {
    /// Priority rank for sorting (higher = more important).
    /// Note rank 0 — informal content never outranks deliberate types.
    pub fn priority_rank(&self) -> u8 {
        match self {
            PageType::Task => 7,
            PageType::Spec => 6,
            PageType::Pattern => 5,
            PageType::Concept => 4,
            PageType::Decision => 3,
            PageType::Howto => 2,
            PageType::Reference => 1,
            PageType::Note => 0,
        }
    }
}

// ─── Memory Entry ───────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MemoryEntry {
    pub id: String,
    pub title: String,
    pub content: String,
    pub tags: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
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

// ─── Decision-specific fields ───────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DecisionEntry {
    pub context: String,
    pub options: Vec<String>,
    pub rationale: String,
    pub outcome: String,
}

// ─── Pattern-specific fields ────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PatternInfo {
    pub when_to_use: String,
    pub example: String,
}

// ─── Acceptance Criterion ───────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AcceptanceCriterion {
    pub text: String,
    pub checked: bool,
}

// ─── Wiki Page Metadata ─────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WikiPageMeta {
    pub id: String,
    pub title: String,
    pub page_type: PageType,
    pub tags: Vec<String>,
    pub status: PageStatus,
    pub priority: Option<Priority>,
    pub confidence: Option<Confidence>,
    pub assignee: Option<String>,
    pub aliases: Vec<String>,
    pub superseded_by: Option<String>,
    pub version: Option<String>,
    pub sources: Vec<String>,
    // Per-type structured fields
    pub acceptance_criteria: Vec<AcceptanceCriterion>,
    pub estimate: Option<u32>,
    pub functional_requirements: Vec<FunctionalRequirement>,
    pub non_functional_requirements: Vec<NonFunctionalRequirement>,
    pub general_goals: Vec<GeneralGoal>,
    pub stakeholders: Vec<String>,
    pub decision: Option<DecisionEntry>,
    pub pattern: Option<PatternInfo>,
    pub prerequisites: Vec<String>,
    pub difficulty: Option<String>,
    pub source_url: Option<String>,
    // Parent task for subtasks
    pub parent: Option<String>,
    // Relationships as string list
    pub relates_to: Vec<String>,
    // Path & timestamps
    pub path: PathBuf,
    pub created_at: String,
    pub updated_at: String,
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
        assert_eq!(PageType::Howto.priority_rank(), 2);
        assert_eq!(PageType::Reference.priority_rank(), 1);
    }

    #[test]
    fn test_edge_type_priority() {
        assert!(EdgeType::Extends.priority() > EdgeType::RelatesTo.priority());
        assert!(EdgeType::Implements.priority() > EdgeType::References.priority());
        assert_eq!(EdgeType::DependsOn.priority(), EdgeType::RequiredBy.priority());
    }
}
