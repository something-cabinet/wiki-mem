use std::collections::BTreeMap;

use serde::Serialize;

use super::migration_note_model::MigrationNote;
use super::page_record_plan_model::PageRecordPlan;
use super::parse_failure_model::ParseFailure;
use super::skip_reason_model::SkipReason;

#[derive(Clone, Debug, Default, Serialize)]
pub struct RecordMigrationPlan {
    pub scanned: usize,
    pub would_convert: BTreeMap<String, usize>,
    pub skipped: BTreeMap<String, BTreeMap<String, usize>>,
    pub excluded: BTreeMap<String, usize>,
    pub already_record: usize,
    pub parse_failures: Vec<ParseFailure>,
    pub notes: Vec<MigrationNote>,
    pub pages: Vec<PageRecordPlan>,
}

impl RecordMigrationPlan {
    pub fn count_convert(&mut self, page_type: &str) {
        increment(&mut self.would_convert, page_type);
    }

    pub fn count_skip(&mut self, page_type: &str, reason: &SkipReason) {
        let bucket = self.skipped.entry(page_type.to_owned()).or_default();
        increment(bucket, reason.kind());
    }

    pub fn count_excluded(&mut self, label: &str) {
        increment(&mut self.excluded, label);
    }

    pub fn would_convert_total(&self) -> usize {
        self.would_convert.values().sum()
    }
}

fn increment(map: &mut BTreeMap<String, usize>, key: &str) {
    map.entry(key.to_owned())
        .and_modify(|count| *count = count.saturating_add(1))
        .or_insert(1);
}
