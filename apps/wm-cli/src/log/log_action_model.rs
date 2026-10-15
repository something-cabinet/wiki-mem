use clap::Subcommand;

#[derive(Subcommand)]
pub enum LogAction {
    Recent {
        #[arg(long, default_value = "20")]
        count: usize,
        #[arg(long)]
        json: bool,
    },

    Since {
        marker: String,
        #[arg(long)]
        json: bool,
    },

    Filter {
        text: String,
        #[arg(long)]
        json: bool,
    },
}
