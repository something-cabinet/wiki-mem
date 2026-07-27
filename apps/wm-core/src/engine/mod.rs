pub use wm_engine::status::{Confidence, MemoryStatus, PageStatus, Priority};
pub use wm_engine::{
    AcceptanceCriterion, AuditEvent, DecisionData, EdgeType, FunctionalRequirement, GeneralGoal,
    GraphSnapshot, MemoryData, MemoryEntry, MemoryLayer, NonFunctionalRequirement, Page, PageType,
    PatternData, RuleCategory, RuleData, SectionDoc, SourceEntry, SourceState, SpecData, TaskData,
    TemplateAction, TemplateConfig, TemplatePrompt, TimeEntry, WikiPageContent, WikiPageMeta,
};

pub mod engine_state_mediator;
pub mod index_scheduler_service;
pub mod main_engine_factory;
pub mod write_channel_proxy;

pub use engine_state_mediator::EngineState;
pub use index_scheduler_service::IndexScheduler;
pub use main_engine_factory::MainEngine;
pub use write_channel_proxy::{WriteChannel, WriteOp};

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
        assert!(EdgeType::DependsOn.priority() > EdgeType::References.priority());
        assert_eq!(EdgeType::PartOf.priority(), EdgeType::Supersedes.priority());
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

    #[test]
    fn test_allowed_statuses_task() {
        assert_eq!(
            PageType::Task.allowed_statuses(),
            &[
                PageStatus::Todo,
                PageStatus::InProgress,
                PageStatus::InReview,
                PageStatus::Done,
                PageStatus::Blocked,
                PageStatus::Cancelled,
            ]
        );
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
}
