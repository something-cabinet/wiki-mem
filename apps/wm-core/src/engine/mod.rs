//! Core engine types — data structures used across the entire wiki memory engine.
//!
//! This module defines all domain types ([`EdgeType`], [`PageType`], [`WikiPageMeta`],
//! [`MemoryEntry`], etc.) and re-exports key types from submodules:
//! - [`EngineState`] — central runtime state
//! - [`WriteChannel`] — sequential file I/O
//! - [`IndexScheduler`] — debounced rebuild scheduler
//! - [`MainEngine`] — bootstrap and lifecycle

pub use crate::status::{Confidence, MemoryStatus, PageStatus, Priority};

pub mod write_channel;
pub mod scheduler;
pub mod state;
pub mod main;
pub mod edge_type;
pub mod page_type;
pub mod memory;
pub mod time_entry;
pub mod audit_event;
pub mod relation;
pub(crate) mod page;
pub(crate) mod page_data;
pub(crate) mod source;
pub(crate) mod template;

pub use edge_type::EdgeType;
pub use page_type::PageType;
pub use memory::{MemoryLayer, MemoryEntry};
pub use time_entry::TimeEntry;
pub use audit_event::AuditEvent;
pub use page::{SectionDoc, WikiPageContent, WikiPageMeta, Page};
pub use page_data::{TaskData, SpecData, DecisionData, PatternData, MemoryData, RuleData, RuleCategory, AcceptanceCriterion, FunctionalRequirement, NonFunctionalRequirement, GeneralGoal, GraphSnapshot};
pub use source::{SourceState, SourceEntry};
pub use template::{TemplatePrompt, TemplateAction, TemplateConfig};

pub use state::EngineState;
pub use write_channel::{WriteChannel, WriteOp};
pub use scheduler::IndexScheduler;
pub use main::MainEngine;
pub(crate) use relation::parse_edge_type_flexible;

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
        use std::path::PathBuf;
        let wpm = WikiPageMeta {
            id: "test".into(), title: "T".into(), tags: vec![], status: PageStatus::Todo,
            published: false, priority: None, confidence: None, assignee: None,
            aliases: vec![], superseded_by: None, version: None, sources: vec![],
            parent: None, relates_to: vec![], path: PathBuf::new(),
            created_at: "".into(), updated_at: "".into(), page_type: PageType::Task,
            order: None, task_data: None, spec_data: None, decision_data: None,
            pattern_data: None, memory_data: None, rule_data: None,
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
        use std::path::PathBuf;
        let mut page = Page::Concept {
            meta: WikiPageMeta {
                id: "test".into(), title: "Original".into(), tags: vec![],
                status: PageStatus::Draft, published: false, priority: None,
                confidence: None, assignee: None, aliases: vec![],
                superseded_by: None, version: None, sources: vec![],
                parent: None, relates_to: vec![], path: PathBuf::new(),
                created_at: "".into(), updated_at: "".into(), page_type: PageType::Concept,
                order: None, task_data: None, spec_data: None, decision_data: None,
                pattern_data: None, memory_data: None, rule_data: None,
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
