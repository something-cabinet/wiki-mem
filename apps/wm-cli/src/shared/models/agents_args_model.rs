use clap::Args;

#[derive(Args)]
pub struct AgentsArgs {
    #[arg(long)]
    pub sync: bool,
    #[arg(long)]
    pub global: bool,
}
