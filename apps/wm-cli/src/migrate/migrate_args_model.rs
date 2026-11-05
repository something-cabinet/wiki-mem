use clap::Args;

#[derive(Args)]
pub struct MigrateArgs {
    #[arg(long)]
    pub dry_run: bool,
    #[arg(long)]
    pub json: bool,
}
