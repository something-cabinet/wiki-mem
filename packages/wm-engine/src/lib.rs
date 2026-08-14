pub mod engine_state;
pub mod helpers;
pub mod models;
pub mod status;

pub use engine_state::EngineState;
pub use helpers::*;
pub use models::*;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::status::PageStatus;

    #[test]
    fn test_page_type_priority_rank() {
        assert_eq!(PageType::Task.priority_rank(), 7);
        assert_eq!(PageType::Spec.priority_rank(), 6);
        assert_eq!(PageType::Pattern.priority_rank(), 5);
        assert_eq!(PageType::Concept.priority_rank(), 4);
        assert_eq!(PageType::Decision.priority_rank(), 3);
        assert_eq!(PageType::Core.priority_rank(), 9);
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
    fn test_edge_provenance_wire_contract() {
        assert_eq!(EdgeProvenance::Explicit.as_str(), "explicit");
        assert_eq!(EdgeProvenance::Derived.as_str(), "derived");
        assert_eq!(EdgeProvenance::Ambiguous.as_str(), "ambiguous");

        assert_eq!(
            serde_json::to_value(EdgeProvenance::Explicit).unwrap(),
            serde_json::json!("explicit")
        );
        assert_eq!(
            serde_json::to_value(EdgeProvenance::Derived).unwrap(),
            serde_json::json!("derived")
        );
        assert_eq!(
            serde_json::to_value(EdgeProvenance::Ambiguous).unwrap(),
            serde_json::json!("ambiguous")
        );

        assert_eq!(EdgeProvenance::Explicit.factor(), 1.0);
        assert_eq!(EdgeProvenance::Derived.factor(), 0.5);
        assert_eq!(EdgeProvenance::Ambiguous.factor(), 0.25);
    }

    #[test]
    fn test_graph_edge_shape() {
        let edge = GraphEdge::new(EdgeType::References, EdgeProvenance::Ambiguous);
        assert_eq!(edge.edge_type, EdgeType::References);
        assert_eq!(edge.provenance, EdgeProvenance::Ambiguous);
        assert_eq!(edge.priority(), EdgeType::References.priority());
        assert_eq!(edge.provenance_factor(), 0.25);
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
        assert_eq!(PageType::Core.as_str(), "core");
        assert_eq!(PageType::Memory.as_str(), "memory");
        assert_eq!(PageType::Howto.as_str(), "howto");
        assert_eq!(PageType::Reference.as_str(), "reference");
        assert_eq!(PageType::Note.as_str(), "note");
    }
}
