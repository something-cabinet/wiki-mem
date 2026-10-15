use clap::Subcommand;

#[derive(Subcommand)]
pub enum ModelAction {
    Download {
        name: String,
        #[arg(long)]
        json: bool,
    },
    List {
        #[arg(long)]
        json: bool,
    },
    Status {
        #[arg(long)]
        json: bool,
    },
    Remove {
        name: String,
        #[arg(long)]
        json: bool,
    },
}
