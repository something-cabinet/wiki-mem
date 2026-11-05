use wm_constants::{WIKI_DIR, WM_DIR};
use wm_migrate::{plan_record_migration, RecordMigrationPlan};

use crate::migrate::{
    apply_with_version_history, print_migration, run_memory_step, MemoryStep, MigrateArgs,
};
use crate::shared::services::project_root;
use crate::shared::traits::Command;

impl Command for MigrateArgs {
    async fn run(self) -> Result<(), anyhow::Error> {
        run_migration(self.dry_run, self.json)
    }
}

fn run_migration(dry_run: bool, json: bool) -> Result<(), anyhow::Error> {
    let root = project_root();
    let memory = run_memory_step(&root, dry_run)?;
    let wiki_dir = root.join(WM_DIR).join(WIKI_DIR);
    let plan = plan_record_migration(&wiki_dir);
    let written = match dry_run {
        true => 0,
        false => apply_with_version_history(&root.join(WM_DIR), &wiki_dir, &plan)
            .map_err(|error| anyhow::anyhow!("Migration apply failed: {}", error))?,
    };
    print_migration(&memory, &plan, dry_run, written, json)?;
    let unmigrated = unmigrated_total(&memory, &plan);
    if unmigrated > 0 {
        anyhow::bail!("{unmigrated} item(s) could not be migrated");
    }
    Ok(())
}

fn unmigrated_total(memory: &MemoryStep, plan: &RecordMigrationPlan) -> usize {
    memory.failed.len().saturating_add(plan.skipped_total())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn memory_failures_and_skipped_pages_both_count() {
        let mut plan = RecordMigrationPlan::default();
        plan.count_skip("concept", &wm_migrate::SkipReason::EmptyState);
        let memory = MemoryStep {
            failed: vec![wm_core::page::MemoryImportFailure {
                file: "bad.json".to_owned(),
                reason: "invalid json".to_owned(),
            }],
            ..MemoryStep::default()
        };

        assert_eq!(unmigrated_total(&memory, &plan), 2);
    }

    #[test]
    fn a_clean_run_has_nothing_unmigrated() {
        assert_eq!(
            unmigrated_total(&MemoryStep::default(), &RecordMigrationPlan::default()),
            0
        );
    }
}
