pub mod edge_type_model;
pub mod page_type_model;
pub mod time_entry_model;
pub mod audit_event_model;
pub mod memory;
pub mod page;
pub mod page_data;
pub mod source;
pub mod template;

// Explicit re-exports (replaced glob re-exports to avoid
// #[allow(ambiguous_glob_reexports)] under Rust 2024 edition).
pub use edge_type_model::EdgeType;
pub use page_type_model::PageType;
pub use time_entry_model::TimeEntry;
pub use audit_event_model::AuditEvent;
pub use memory::{MemoryEntry, MemoryLayer};
pub use page::{Page, SectionDoc, WikiPageContent, WikiPageMeta};
pub use page_data::{
    AcceptanceCriterion, DecisionData, FunctionalRequirement, GeneralGoal, GraphSnapshot,
    MemoryData, NonFunctionalRequirement, PatternData, RuleCategory, RuleData, SpecData, TaskData,
};
pub use source::{SourceEntry, SourceState};
pub use template::{TemplateAction, TemplateConfig, TemplatePrompt};
