use clap::Subcommand;

#[derive(Subcommand)]
pub enum SourceAction {
    List {
        #[arg(long)]
        state: Option<String>,
        #[arg(long)]
        json: bool,
    },

    Status {
        id: String,
        #[arg(long)]
        json: bool,
    },

    Remove {
        id: String,
        #[arg(long)]
        json: bool,
    },

    Discover {
        #[arg(long)]
        json: bool,
    },
}
