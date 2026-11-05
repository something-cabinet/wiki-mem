use std::path::Path;

use crate::migrate::MemoryStep;
use crate::shared::services::{count_legacy_memory_entries, load_engine};

pub fn run_memory_step(root: &Path, dry_run: bool) -> Result<MemoryStep, anyhow::Error> {
    let pending = count_legacy_memory_entries(root);
    if dry_run || pending == 0 {
        return Ok(MemoryStep {
            pending,
            ..MemoryStep::default()
        });
    }
    let engine = load_engine(root);
    let report = wm_core::page::import_legacy_memory(&engine.state)
        .map_err(|error| anyhow::anyhow!("Memory import failed: {}", error))?;
    Ok(MemoryStep {
        imported: report.imported,
        pending: 0,
        failed: report.failed,
    })
}
