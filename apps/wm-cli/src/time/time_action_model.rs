use clap::Subcommand;

#[derive(Subcommand)]
pub enum TimeAction {
    Start {
        id: String,
        #[arg(long)]
        json: bool,
    },

    Stop {
        id: String,
        #[arg(long)]
        json: bool,
    },

    Add {
        id: String,
        duration: String,
        #[arg(long)]
        json: bool,
    },

    Report {
        #[arg(long)]
        json: bool,
    },
}
