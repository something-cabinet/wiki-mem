use clap::Args;

#[derive(Args)]
pub struct StatusArgs {
    #[arg(long)]
    pub json: bool,
}
