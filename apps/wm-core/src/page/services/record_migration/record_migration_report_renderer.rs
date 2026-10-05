use std::fmt::Write;

use super::record_migration_plan_model::RecordMigrationPlan;

const LABEL_WIDTH: usize = 12;

pub fn render_migration_report(plan: &RecordMigrationPlan) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "Record migration report (dry-run)");
    let _ = writeln!(out, "=================================");
    let _ = writeln!(out);
    let _ = writeln!(out, "Scanned: {} markdown files", plan.scanned);
    render_would_convert(&mut out, plan);
    render_skipped(&mut out, plan);
    render_excluded(&mut out, plan);
    render_parse_failures(&mut out, plan);
    render_notes(&mut out, plan);
    out
}

fn render_would_convert(out: &mut String, plan: &RecordMigrationPlan) {
    let total = plan.would_convert_total();
    let _ = writeln!(out);
    let _ = writeln!(out, "Would convert: {total} page(s)");
    if plan.would_convert.is_empty() {
        let _ = writeln!(out, "  (none)");
    }
    for (page_type, count) in &plan.would_convert {
        let _ = writeln!(out, "  {page_type:<LABEL_WIDTH$} {count}");
    }
}

fn render_skipped(out: &mut String, plan: &RecordMigrationPlan) {
    let _ = writeln!(out);
    let _ = writeln!(out, "Skipped (record-bearing): {}", skip_total(plan));
    if plan.skipped.is_empty() {
        let _ = writeln!(out, "  (none)");
    }
    for (page_type, reasons) in &plan.skipped {
        for (reason, count) in reasons {
            let _ = writeln!(out, "  {page_type:<LABEL_WIDTH$} {reason:<LABEL_WIDTH$} {count}");
        }
    }
    for page in &plan.pages {
        let super::page_record_outcome_model::PageRecordOutcome::Skip { reason } = &page.outcome
        else {
            continue;
        };
        let _ = writeln!(
            out,
            "    {} [{}] {}",
            page.path,
            page.page_type,
            reason.detail()
        );
    }
}

fn render_excluded(out: &mut String, plan: &RecordMigrationPlan) {
    let _ = writeln!(out);
    let _ = writeln!(out, "Excluded (not record-bearing): {}", excluded_total(plan));
    if plan.excluded.is_empty() {
        let _ = writeln!(out, "  (none)");
    }
    for (label, count) in &plan.excluded {
        let _ = writeln!(out, "  {label:<LABEL_WIDTH$} {count}");
    }
}

fn render_parse_failures(out: &mut String, plan: &RecordMigrationPlan) {
    let _ = writeln!(out);
    let _ = writeln!(
        out,
        "Already records (no-op): {}",
        plan.already_record
    );
    let _ = writeln!(out);
    let _ = writeln!(out, "Parse failures: {}", plan.parse_failures.len());
    for failure in &plan.parse_failures {
        let _ = writeln!(out, "  {}: {}", failure.path, failure.error);
    }
}

fn render_notes(out: &mut String, plan: &RecordMigrationPlan) {
    let _ = writeln!(out);
    let _ = writeln!(out, "Notes: {}", plan.notes.len());
    for note in &plan.notes {
        let _ = writeln!(out, "  {}: {}", note.path, note.note);
    }
}

fn skip_total(plan: &RecordMigrationPlan) -> usize {
    plan.skipped
        .values()
        .flat_map(|reasons| reasons.values())
        .sum()
}

fn excluded_total(plan: &RecordMigrationPlan) -> usize {
    plan.excluded.values().sum()
}
