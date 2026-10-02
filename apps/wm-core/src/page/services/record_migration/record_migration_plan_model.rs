use std::collections::BTreeMap;

use serde::Serialize;

use super::migration_note_model::MigrationNote;
use super::page_record_plan_model::PageRecordPlan;
use super::parse_failure_model::ParseFailure;
use super::skip_reason_model::SkipReason;

/// The complete, read-only plan produced by the migration planner.
#[derive(Clone, Debug, Default, Serialize)]
pub struct RecordMigrationPlan {
    /// Every `.md` file visited under the wiki root.
    pub scanned: usize,
    /// Would-convert counts keyed by record-bearing type name.
    pub would_convert: BTreeMap<String, usize>,
    /// Skipped counts keyed by type name, then by [`SkipReason::kind`].
    pub skipped: BTreeMap<String, BTreeMap<String, usize>>,
    /// Excluded page counts keyed by type or directory label.
    pub excluded: BTreeMap<String, usize>,
    /// Pages whose bodies already satisfy `schema_version: 1`.
    pub already_record: usize,
    /// Hard parse failures encountered while scanning.
    pub parse_failures: Vec<ParseFailure>,
    /// Non-fatal observations (escaped newlines preserved, `relates_to` union).
    pub notes: Vec<MigrationNote>,
    /// Per-page decisions, in sorted path order.
    pub pages: Vec<PageRecordPlan>,
}

impl RecordMigrationPlan {
    /// Adds a would-convert count for `page_type`.
    pub fn count_convert(&mut self, page_type: &str) {
        increment(&mut self.would_convert, page_type);
    }

    /// Adds a skipped count for `page_type` under the given reason.
    pub fn count_skip(&mut self, page_type: &str, reason: &SkipReason) {
        let bucket = self.skipped.entry(page_type.to_owned()).or_default();
        increment(bucket, reason.kind());
    }

    /// Adds an excluded count for `label`.
    pub fn count_excluded(&mut self, label: &str) {
        increment(&mut self.excluded, label);
    }

    /// Total pages that would be converted across every type.
    pub fn would_convert_total(&self) -> usize {
        self.would_convert.values().sum()
    }
}

fn increment(map: &mut BTreeMap<String, usize>, key: &str) {
    map.entry(key.to_owned())
        .and_modify(|count| *count = count.saturating_add(1))
        .or_insert(1);
}
