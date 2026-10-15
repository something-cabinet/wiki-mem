use clap::Args;

#[derive(Args)]
pub struct DecideArgs {
    #[arg(long)]
    pub page: Option<String>,
    #[arg(long)]
    pub state: Option<String>,
    #[arg(long = "type")]
    pub page_type: Option<String>,
    #[arg(long)]
    pub json: bool,
}
