use clap::Subcommand;

#[derive(Subcommand)]
pub enum SearchAction {
    Query {
        query: String,
        #[arg(long)]
        mode: Option<String>,
        #[arg(long)]
        r#type: Option<String>,
        #[arg(long, default_value = "10")]
        limit: usize,
        #[arg(long)]
        json: bool,
    },

    Retrieve {
        query: String,
        #[arg(long, default_value = "8192")]
        token_budget: usize,
        #[arg(long)]
        json: bool,
    },

    Resolve {
        query: String,
        #[arg(long)]
        json: bool,
    },
}
