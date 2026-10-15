use clap::Args;

#[derive(Args)]
pub struct SetupArgs {
    pub platform: String,
    #[arg(long)]
    pub strict: bool,
}
