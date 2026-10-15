use std::path::PathBuf;

use crate::agents::AgentsArgs;
use crate::shared::services::sync_agent_files;
use crate::shared::traits::Command;

impl Command for AgentsArgs {
    async fn run(self) -> Result<(), anyhow::Error> {
        run_agent_sync(self.global)
    }
}

fn run_agent_sync(global: bool) -> Result<(), anyhow::Error> {
    let root = resolve_sync_root(global);
    let platforms: Vec<String> = Vec::new();
    sync_agent_files(&root, &platforms, false)?;
    println!("Agent instruction files synced.");
    Ok(())
}

fn resolve_sync_root(global: bool) -> PathBuf {
    if !global {
        return PathBuf::from(".");
    }
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .unwrap_or_else(|_| ".".into());
    PathBuf::from(home)
}
