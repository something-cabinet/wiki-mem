use wm_migrate::{render_migration_report, RecordMigrationPlan};

use crate::migrate::MemoryStep;

pub fn print_migration(
    memory: &MemoryStep,
    plan: &RecordMigrationPlan,
    dry_run: bool,
    written: usize,
    json: bool,
) -> Result<(), anyhow::Error> {
    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&json_report(memory, plan, dry_run, written))?
        );
        return Ok(());
    }
    print_memory(memory, dry_run);
    println!();
    print!("{}", render_migration_report(plan, dry_run));
    if !dry_run {
        println!();
        println!("Wrote {written} page(s).");
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

fn json_report(
    memory: &MemoryStep,
    plan: &RecordMigrationPlan,
    dry_run: bool,
    written: usize,
) -> serde_json::Value {
    let failed: Vec<serde_json::Value> = memory
        .failed
        .iter()
        .map(|failure| serde_json::json!({ "file": failure.file, "reason": failure.reason }))
        .collect();
    serde_json::json!({
        "dry_run": dry_run,
        "written": written,
        "memory": {
            "imported": memory.imported,
            "pending": memory.pending,
            "failed": failed,
        },
        "records": plan,
    })
}
