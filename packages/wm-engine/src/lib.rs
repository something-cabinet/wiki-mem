pub mod edge_type;
pub mod page_type;
pub mod relation;
pub mod time_entry;
pub mod audit_event;
pub mod memory;
pub mod page_data;
pub mod source;
pub mod template;
pub(crate) mod page;

pub use edge_type::EdgeType;
pub use page_data::GraphSnapshot;
pub use page_type::PageType;
pub use memory::{MemoryLayer, MemoryEntry};
pub use time_entry::TimeEntry;
pub use audit_event::AuditEvent;
pub use page::{SectionDoc, WikiPageContent, WikiPageMeta, Page};
pub use page_data::{TaskData, SpecData, DecisionData, PatternData, MemoryData, RuleData, RuleCategory, AcceptanceCriterion, FunctionalRequirement, NonFunctionalRequirement, GeneralGoal};
pub use source::{SourceState, SourceEntry};
pub use template::{TemplatePrompt, TemplateAction, TemplateConfig};

pub use relation::parse_edge_type_flexible;

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use wm_status::{PageStatus, Priority};

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
        assert_eq!(PageType::Task.allowed_statuses(), &[
            PageStatus::Todo, PageStatus::InProgress, PageStatus::InReview,
            PageStatus::Done, PageStatus::Blocked, PageStatus::Cancelled,
        ]);
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
