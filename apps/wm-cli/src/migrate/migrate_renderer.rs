use wm_migrate::{render_migration_report, RecordMigrationPlan};

use crate::migrate::{ManagedApplied, ManagedPlan, ManagedStatus, MemoryStep};

pub struct MigrationReport<'a> {
    pub memory: &'a MemoryStep,
    pub plan: &'a RecordMigrationPlan,
    pub managed: &'a ManagedPlan,
    pub applied: &'a ManagedApplied,
    pub dry_run: bool,
    pub written: usize,
}

pub fn print_migration(report: &MigrationReport<'_>, json: bool) -> Result<(), anyhow::Error> {
    if json {
        println!("{}", serde_json::to_string_pretty(&json_report(report))?);
        return Ok(());
    }
    print_memory(report.memory, report.dry_run);
    println!();
    print_managed(report);
    println!();
    print!("{}", render_migration_report(report.plan, report.dry_run));
    if !report.dry_run {
        println!();
        println!("Wrote {} page(s).", report.written);
    }
    Ok(())
}

fn print_memory(memory: &MemoryStep, dry_run: bool) {
    if dry_run {
        println!(
            "Legacy memory: {} entr(ies) would be imported",
            memory.pending
        );
    } else {
        println!("Legacy memory: {} entr(ies) imported", memory.imported);
    }
    for failure in &memory.failed {
        println!("  failed {}: {}", failure.file, failure.reason);
    }
}

fn print_managed(report: &MigrationReport<'_>) {
    let managed = report.managed;
    let verb = match report.dry_run {
        true => "would be",
        false => "were",
    };
    println!(
        "Managed files: {} up to date, {} {verb} recorded, {} {verb} updated, {} edited (kept), {} unmanaged (kept)",
        managed.count(ManagedStatus::UpToDate),
        managed.count(ManagedStatus::Record),
        managed.count(ManagedStatus::Update),
        managed.count(ManagedStatus::Edited),
        managed.count(ManagedStatus::Unmanaged),
    );
    for file in &managed.files {
        let kept = matches!(
            file.status,
            ManagedStatus::Edited | ManagedStatus::Unmanaged
        );
        if kept {
            println!("  kept {}: {:?}", file.path, file.status);
        }
    }
}

fn json_report(report: &MigrationReport<'_>) -> serde_json::Value {
    let failed: Vec<serde_json::Value> = report
        .memory
        .failed
        .iter()
        .map(|failure| serde_json::json!({ "file": failure.file, "reason": failure.reason }))
        .collect();
    let managed_files: Vec<serde_json::Value> = report
        .managed
        .files
        .iter()
        .map(|file| serde_json::json!({ "path": file.path, "status": file.status }))
        .collect();
    serde_json::json!({
        "dry_run": report.dry_run,
        "written": report.written,
        "memory": {
            "imported": report.memory.imported,
            "pending": report.memory.pending,
            "failed": failed,
        },
        "managed": {
            "updated": report.applied.updated,
            "recorded": report.applied.recorded,
            "files": managed_files,
        },
        "records": report.plan,
    })
}
