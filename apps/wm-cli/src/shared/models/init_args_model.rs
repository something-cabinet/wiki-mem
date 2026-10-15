use clap::Args;
use std::path::PathBuf;

#[derive(Args)]
pub struct InitArgs {
    #[arg(long)]
    pub project: Option<PathBuf>,

    #[arg(long)]
    pub platform: Option<String>,

    #[arg(long)]
    pub no_wizard: bool,
}
