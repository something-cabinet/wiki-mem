//! Read-only planning and (gated) application of the typed-decision record
//! migration.
//!
//! [`plan_record_migration`] scans a wiki directory and classifies every page
//! against the record-bearing types and the TD-02 exclusion list. It never
//! writes. [`apply_wiki_record_migration`] performs the destructive rewrite and
//! is only reachable through the explicit `--apply` CLI flag.

pub mod frontmatter_prepass;
pub mod migration_note_model;
pub mod page_record_outcome_model;
pub mod page_record_plan_model;
pub mod parse_failure_model;
pub mod record_body_builder;
pub mod record_migration_applier;
pub mod record_migration_filter_model;
pub mod record_migration_plan_model;
pub mod record_migration_planner;
pub mod record_migration_report_renderer;
pub mod skip_reason_model;

pub use migration_note_model::MigrationNote;
pub use page_record_outcome_model::PageRecordOutcome;
pub use page_record_plan_model::PageRecordPlan;
pub use parse_failure_model::ParseFailure;
pub use record_body_builder::build_record_body;
pub use record_migration_applier::{apply_wiki_record_migration, replace_page_body};
pub use record_migration_filter_model::RecordMigrationFilter;
pub use record_migration_plan_model::RecordMigrationPlan;
pub use record_migration_planner::{plan_record_migration, plan_record_migration_filtered};
pub use record_migration_report_renderer::render_migration_report;
pub use skip_reason_model::SkipReason;

