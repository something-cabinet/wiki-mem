use clap::Subcommand;

#[derive(Subcommand)]
pub enum ConfigAction {
    Get {
        key: String,
        #[arg(long)]
        json: bool,
    },
    Set {
        key: String,
        value: String,
        #[arg(long)]
        json: bool,
    },
    List {
        #[arg(long)]
        json: bool,
    },
}
