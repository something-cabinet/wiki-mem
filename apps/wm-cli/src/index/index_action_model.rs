use clap::Subcommand;
use wm_constants::EMBED_BATCH_SIZE;

#[derive(Subcommand)]
pub enum IndexAction {
    Rebuild {
        #[arg(long)]
        skip_embed: bool,
        #[arg(long, default_value_t = EMBED_BATCH_SIZE)]
        batch_size: usize,
        #[arg(long)]
        since: Option<String>,
    },

    Embed {
        #[arg(long, default_value_t = EMBED_BATCH_SIZE)]
        batch_size: usize,
        #[arg(long)]
        force: bool,
    },

    Code {
        #[arg(long)]
        skip_hash_check: bool,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;
    use wm_constants::EMBED_BATCH_SIZE;

    #[derive(Parser)]
    struct Harness {
        #[command(subcommand)]
        action: IndexAction,
    }

    fn parsed_action(args: &[&str]) -> IndexAction {
        Harness::try_parse_from(args)
            .unwrap_or_else(|error| panic!("{error}"))
            .action
    }

    #[test]
    fn rebuild_batch_size_defaults_to_the_shared_constant() {
        let IndexAction::Rebuild { batch_size, .. } = parsed_action(&["wm", "rebuild"]) else {
            panic!("expected rebuild");
        };
        assert_eq!(batch_size, EMBED_BATCH_SIZE);
    }

    #[test]
    fn embed_batch_size_defaults_to_the_shared_constant() {
        let IndexAction::Embed { batch_size, .. } = parsed_action(&["wm", "embed"]) else {
            panic!("expected embed");
        };
        assert_eq!(batch_size, EMBED_BATCH_SIZE);
    }

    #[test]
    fn explicit_batch_size_is_respected() {
        let IndexAction::Rebuild { batch_size, .. } =
            parsed_action(&["wm", "rebuild", "--batch-size", "3"])
        else {
            panic!("expected rebuild");
        };
        assert_eq!(batch_size, 3);
    }
}
