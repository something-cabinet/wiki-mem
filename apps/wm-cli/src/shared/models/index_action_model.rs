use clap::Subcommand;

#[derive(Subcommand)]
pub enum IndexAction {
    Rebuild {
        #[arg(long)]
        skip_embed: bool,
        #[arg(long, default_value = "32")]
        batch_size: usize,
        #[arg(long)]
        since: Option<String>,
    },

    Embed {
        #[arg(long, default_value = "32")]
        batch_size: usize,
        #[arg(long)]
        force: bool,
    },

    Code {
        #[arg(long)]
        skip_hash_check: bool,
    },
}
