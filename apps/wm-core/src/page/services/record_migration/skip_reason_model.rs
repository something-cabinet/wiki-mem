use serde::Serialize;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case", tag = "reason")]
pub enum SkipReason {
    MissingFrontmatter,
    UnparseableFrontmatter { block: usize },
    DuplicateFrontmatterConflict { key: String },
    EmptyState,
    InvalidRecordBody { error: String },
    LimitReached,
}

impl SkipReason {
    pub fn kind(&self) -> &'static str {
        match self {
            SkipReason::MissingFrontmatter => "missing-frontmatter",
            SkipReason::UnparseableFrontmatter { .. } => "unparseable-frontmatter",
            SkipReason::DuplicateFrontmatterConflict { .. } => "duplicate-frontmatter-conflict",
            SkipReason::InvalidRecordBody { .. } => "invalid-record-body",
            SkipReason::EmptyState => "empty-state",
            SkipReason::LimitReached => "limit-reached",
        }
    }

    pub fn detail(&self) -> String {
        match self {
            SkipReason::MissingFrontmatter => "no leading frontmatter block".to_owned(),
            SkipReason::UnparseableFrontmatter { block } => {
                format!("frontmatter block {block} is not a mapping")
            }
            SkipReason::DuplicateFrontmatterConflict { key } => {
                format!("duplicate frontmatter disagrees on `{key}`")
            }
            SkipReason::InvalidRecordBody { error } => {
                format!("generated record body did not parse: {error}")
            }
            SkipReason::EmptyState => "normalized body is empty".to_owned(),
            SkipReason::LimitReached => "conversion limit already reached".to_owned(),
        }
    }
}
