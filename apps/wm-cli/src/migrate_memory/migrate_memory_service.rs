use crate::migrate_memory::MigrateMemoryArgs;
use crate::shared::services::create_engine;
use crate::shared::traits::Command;

impl Command for MigrateMemoryArgs {
    async fn run(self) -> Result<(), anyhow::Error> {
        run_memory_migration();
        Ok(())
    }
}

fn run_memory_migration() {
    let (engine, _) = create_engine();
    match wm_core::page::migrate_old_memory_json(&engine.state) {
        Ok(count) => println!("Migrated {} memory entries to wiki pages.", count),
        Err(error) => eprintln!("Migration error: {}", error),
    }
}
