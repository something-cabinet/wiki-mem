use clap::Args;

#[derive(Args)]
pub struct ValidateArgs {
    #[arg(long)]
    pub json: bool,
}
