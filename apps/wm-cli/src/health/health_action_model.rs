use clap::Subcommand;

#[derive(Subcommand)]
pub enum HealthAction {
    Audit {
        #[arg(long)]
        dry_run: bool,

        #[arg(long)]
        fix: bool,

        #[arg(long, default_value = "text")]
        format: String,
    },
}
