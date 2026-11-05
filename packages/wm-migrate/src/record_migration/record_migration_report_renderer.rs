use std::fmt::Write;

use super::record_migration_plan_model::RecordMigrationPlan;

const LABEL_WIDTH: usize = 12;
const STEP_WIDTH: usize = 24;
const MODE_DRY_RUN: &str = "dry-run";
const MODE_APPLIED: &str = "applied";

pub fn render_migration_report(plan: &RecordMigrationPlan, dry_run: bool) -> String {
    let mut out = String::new();
    let mode = if dry_run { MODE_DRY_RUN } else { MODE_APPLIED };
    let _ = writeln!(out, "Record migration report ({mode})");
    let _ = writeln!(out, "=================================");
    let _ = writeln!(out);
    let _ = writeln!(out, "Scanned: {} markdown files", plan.scanned);
    render_would_convert(&mut out, plan, dry_run);
    render_steps(&mut out, plan);
    render_skipped(&mut out, plan);
    render_excluded(&mut out, plan);
    render_parse_failures(&mut out, plan);
    render_notes(&mut out, plan);
    out
}

fn render_would_convert(out: &mut String, plan: &RecordMigrationPlan, dry_run: bool) {
    let verb = if dry_run {
        "Would convert"
    } else {
        "Converted"
    };
    let total = plan.would_convert_total();
    let _ = writeln!(out);
    let _ = writeln!(out, "{verb}: {total} page(s)");
    if plan.would_convert.is_empty() {
        let _ = writeln!(out, "  (none)");
    }
    for (page_type, count) in &plan.would_convert {
        let _ = writeln!(out, "  {page_type:<LABEL_WIDTH$} {count}");
    }
}

fn render_steps(out: &mut String, plan: &RecordMigrationPlan) {
    let _ = writeln!(out);
    let _ = writeln!(out, "Steps:");
    if plan.steps.is_empty() {
        let _ = writeln!(out, "  (none)");
    }
    for (step, count) in &plan.steps {
        let _ = writeln!(out, "  {step:<STEP_WIDTH$} {count}");
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
            let _ = writeln!(
                out,
                "  {page_type:<LABEL_WIDTH$} {reason:<LABEL_WIDTH$} {count}"
            );
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
    let _ = writeln!(
        out,
        "Excluded (not record-bearing): {}",
        excluded_total(plan)
    );
    if plan.excluded.is_empty() {
        let _ = writeln!(out, "  (none)");
    }
    for (label, count) in &plan.excluded {
        let _ = writeln!(out, "  {label:<LABEL_WIDTH$} {count}");
    }
}

fn render_parse_failures(out: &mut String, plan: &RecordMigrationPlan) {
    let _ = writeln!(out);
    let _ = writeln!(out, "Already records (no-op): {}", plan.already_record);
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
    plan.skipped_total()
}

fn excluded_total(plan: &RecordMigrationPlan) -> usize {
    plan.excluded.values().sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn plan_with_one_conversion() -> RecordMigrationPlan {
        let mut plan = RecordMigrationPlan::default();
        plan.count_convert("concept");
        plan.count_step("convert-prose");
        plan
    }

    #[test]
    fn a_dry_run_report_says_would_convert() {
        let report = render_migration_report(&plan_with_one_conversion(), true);

        assert!(report.contains("Record migration report (dry-run)"));
        assert!(report.contains("Would convert: 1 page(s)"));
    }

    #[test]
    fn an_applied_report_says_converted() {
        let report = render_migration_report(&plan_with_one_conversion(), false);

        assert!(report.contains("Record migration report (applied)"));
        assert!(report.contains("Converted: 1 page(s)"));
    }

    #[test]
    fn the_report_lists_each_step_with_its_count() {
        let report = render_migration_report(&plan_with_one_conversion(), true);

        assert!(report.contains("Steps:"));
        assert!(report.contains("convert-prose"));
    }

    #[test]
    fn an_empty_plan_reports_no_steps() {
        let report = render_migration_report(&RecordMigrationPlan::default(), true);

        assert!(report.contains("Steps:\n  (none)"));
    }
}
