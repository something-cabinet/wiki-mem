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

    #[command(
        about = "Update a page from a JSON object on stdin (fields: content, status, tags, title, relates_to, type, extra_frontmatter, notes, append_notes)",
        long_about = "Update a page from a JSON object on stdin.\n\nFields: content, status, tags, title, relates_to, type, extra_frontmatter, notes, append_notes.\n\n`content` must be the full typed-decision record envelope (`schema_version`, `state`, `questions`, `answers`) for record-bearing page types. Omit `content` to leave the existing body untouched. Use --content-file <path> to pass a body without shell quoting."
    )]
    Update {
        id: String,

        #[arg(
            long,
            value_name = "PATH",
            help = "Read the page body from PATH instead of the stdin `content` field"
        )]
        content_file: Option<std::path::PathBuf>,

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
