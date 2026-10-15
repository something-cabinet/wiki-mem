use clap::Subcommand;

#[derive(Subcommand)]
pub enum PageAction {
    Get {
        id: String,
        #[arg(long)]
        json: bool,
    },

    List {
        #[arg(long)]
        json: bool,
    },

    Create {
        path: String,
        title: String,
        #[arg(long)]
        page_type: Option<String>,
        #[arg(long)]
        json: bool,
    },

    Delete {
        id: String,
        #[arg(long)]
        json: bool,
    },

    Update {
        id: String,

        #[arg(long)]
        json: bool,
    },

    Link {
        id: String,
        target: String,
        #[arg(long)]
        edge_type: Option<String>,
        #[arg(long)]
        json: bool,
    },

    Unlink {
        id: String,
        target: String,
        #[arg(long)]
        json: bool,
    },

    MigrateRecords {
        #[arg(long)]
        apply: bool,
        #[arg(long = "only", value_name = "PATH")]
        only: Vec<String>,
        #[arg(long, value_name = "N")]
        limit: Option<usize>,
        #[arg(long)]
        json: bool,
    },
}
