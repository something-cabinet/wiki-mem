use std::path::Path;

use wm_core::error::ToolResult;

use crate::migrate_memory::MigrateMemoryArgs;
use crate::shared::services::{has_legacy_memory_entries, load_engine, project_root};
use crate::shared::traits::Command;

impl Command for MigrateMemoryArgs {
    async fn run(self) -> Result<(), anyhow::Error> {
        run_memory_migration()
    }
}

fn run_memory_migration() -> Result<(), anyhow::Error> {
    let count = migrate_legacy_entries(&project_root())
        .map_err(|error| anyhow::anyhow!("Migration error: {}", error))?;
    println!("Migrated {} memory entries to wiki pages.", count);
    Ok(())
}

fn migrate_legacy_entries(root: &Path) -> ToolResult<usize> {
    if !has_legacy_memory_entries(root) {
        return Ok(0);
    }
    let engine = load_engine(root);
    wm_core::page::migrate_old_memory_json(&engine.state)
}
